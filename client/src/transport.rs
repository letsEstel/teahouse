//! Talking to a relay: libgit2's own ssh (libssh2) for real relays, and a
//! development transport that runs the relay's forced command directly.
//!
//! * `ssh://teahouse@relay.example.org:2222/<house>`: in-process ssh. The
//!   device key authenticates; the relay's host key is pinned on first use
//!   in `<root>/known_hosts`.
//! * `teahouse-local://<house>/<relay-root>` (desktop development only):
//!   runs `teahouse-relay --root <relay-root> shell <device-key>` with the
//!   git command in `SSH_ORIGINAL_COMMAND`, exactly what sshd would run.
//!   Like the relay's `local-ssh`, it proves nothing about identity.

use crate::error::{Code, Error, Result};
use crate::identity::Identity;
use git2::{CertificateCheckStatus, Cred, CredentialType, RemoteCallbacks};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

pub const LOCAL_SCHEME: &str = "teahouse-local";

/// What one fetch or push learned besides its result.
#[derive(Default)]
pub struct Report {
    /// Lines the relay printed (the "remote:" lines of the git program).
    pub remote_lines: Vec<String>,
    /// `(ref, reason)` for every ref the relay refused.
    pub refused: Vec<(String, String)>,
    /// The relay showed a different host key than the pinned one.
    pub host_key_changed: bool,
    partial: String,
}

impl Report {
    fn take_text(&mut self, data: &[u8]) {
        self.partial.push_str(&String::from_utf8_lossy(data));
        while let Some(i) = self.partial.find(['\n', '\r']) {
            let line = self.partial[..i].trim().to_string();
            self.partial.drain(..=i);
            if !line.is_empty() && !self.remote_lines.contains(&line) {
                self.remote_lines.push(line);
            }
        }
    }

    /// The relay's reasons, without git's own progress chatter.
    pub fn reasons(&self) -> Vec<String> {
        let mut v: Vec<String> = self
            .remote_lines
            .iter()
            .chain(std::iter::once(&self.partial))
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !is_progress(l))
            .collect();
        v.dedup();
        v
    }
}

fn is_progress(line: &str) -> bool {
    [
        "Counting objects",
        "Compressing objects",
        "Total ",
        "Enumerating objects",
        "Resolving deltas",
    ]
    .iter()
    .any(|p| line.starts_with(p))
}

/// The URL of a teahouse at a relay: `relay` is `user@host[:port]`, an
/// `ssh://` URL of the relay, or `teahouse-local:<relay-root>`.
pub fn house_url(relay: &str, house: &str) -> Result<String> {
    if let Some(root) = relay.strip_prefix("teahouse-local:") {
        let root = root.trim_start_matches("//");
        if !root.starts_with('/') {
            return Err(Error::invalid(
                "teahouse-local needs an absolute relay root",
            ));
        }
        return Ok(format!("{LOCAL_SCHEME}://{house}{root}"));
    }
    let rest = relay
        .strip_prefix("ssh://")
        .unwrap_or(relay)
        .trim_end_matches('/');
    if rest.is_empty() || rest.contains(['/', ' ']) {
        return Err(Error::invalid(format!("not a relay address: {relay:?}")).with("relay", relay));
    }
    // The ssh user defaults to `teahouse`, as everywhere else: without one,
    // libgit2 asks for a user name and the device key is never offered.
    let user = if rest.contains('@') { "" } else { "teahouse@" };
    Ok(format!("ssh://{user}{rest}/{house}"))
}

/// Remote callbacks for one operation: credentials from the device key,
/// host keys pinned on first use, and the relay's messages collected.
pub fn callbacks<'a>(
    identity: &'a Identity,
    known_hosts: &'a Path,
    report: Rc<RefCell<Report>>,
) -> RemoteCallbacks<'a> {
    let mut cb = RemoteCallbacks::new();
    let mut tried = false;
    cb.credentials(move |_url, user, allowed| {
        if tried
            || !allowed.contains(CredentialType::SSH_MEMORY)
                && !allowed.contains(CredentialType::SSH_KEY)
        {
            return Err(git2::Error::from_str(
                "the relay did not accept this device's key",
            ));
        }
        tried = true;
        let user = user.unwrap_or("teahouse");
        // libssh2 takes keys from memory only with its OpenSSL backend
        // (not on Windows, where it uses WinCNG): elsewhere, the key file.
        if cfg!(unix) {
            let private = identity
                .private_openssh()
                .map_err(|e| git2::Error::from_str(&e.message))?;
            Cred::ssh_key_from_memory(user, Some(&identity.public_line()), &private, None)
        } else {
            Cred::ssh_key(user, None, identity.private_path(), None)
        }
    });
    let r = report.clone();
    cb.certificate_check(move |cert, host| {
        let Some(hk) = cert.as_hostkey() else {
            return Err(git2::Error::from_str(
                "the relay did not show an ssh host key",
            ));
        };
        let Some(hash) = hk.hash_sha256() else {
            return Err(git2::Error::from_str(
                "the relay's host key has no SHA-256 fingerprint",
            ));
        };
        let fp: String = hash.iter().map(|b| format!("{b:02x}")).collect();
        match pin_host_key(known_hosts, host, &fp) {
            Ok(()) => Ok(CertificateCheckStatus::CertificateOk),
            Err(e) => {
                r.borrow_mut().host_key_changed = true;
                Err(git2::Error::from_str(&e.message))
            }
        }
    });
    let r = report.clone();
    cb.sideband_progress(move |data| {
        r.borrow_mut().take_text(data);
        true
    });
    let r = report.clone();
    cb.push_update_reference(move |refname, status| {
        if let Some(s) = status {
            r.borrow_mut()
                .refused
                .push((refname.to_string(), s.to_string()));
        }
        Ok(())
    });
    cb
}

/// Trust on first use: remembers `host`'s fingerprint, and refuses a
/// different one later.
pub(crate) fn pin_host_key(known_hosts: &Path, host: &str, fp: &str) -> Result<()> {
    let text = std::fs::read_to_string(known_hosts).unwrap_or_default();
    for line in text.lines() {
        if let Some((h, f)) = line.split_once(' ') {
            if h == host {
                if f.trim() == fp {
                    return Ok(());
                }
                return Err(Error::new(
                    Code::Offline,
                    format!("the relay {host} shows a different host key than before / Relay 的主机钥匙变了"),
                )
                .with("relay", host)
                .with("reason", "host_key_changed"));
            }
        }
    }
    let mut text = text;
    text.push_str(&format!("{host} {fp}\n"));
    std::fs::write(known_hosts, text)?;
    Ok(())
}

/// Maps a failed fetch or push to the API's errors.
pub fn classify(e: &git2::Error, report: &Report) -> Error {
    let reasons = report.reasons();
    // Over ssh an unknown key fails authentication; a key seated in another
    // teahouse gets the relay's answer for "not here" (law 1: unknown
    // teahouses and teahouses without your seat look the same).
    let no_seat = reasons.iter().any(|r| r.contains("you have no seat"))
        || e.message().contains("you have no seat");
    if no_seat
        || e.code() == git2::ErrorCode::Auth
        || e.message().contains("did not accept this device")
    {
        let mut err = Error::new(
            Code::NotAMember,
            "the relay does not know this device / Relay 不认识这台设备",
        );
        err.reasons = reasons;
        return err;
    }
    if report.host_key_changed {
        return Error::new(
            Code::Offline,
            "the relay shows a different host key than before / Relay 的主机钥匙变了",
        )
        .with("reason", "host_key_changed");
    }
    let mut err = Error::new(
        Code::Offline,
        format!("cannot reach the relay / 连不上 Relay: {}", e.message()),
    );
    err.reasons = reasons;
    err
}

// ---------------------------------------------------------------------------
// The development transport

#[cfg(not(target_os = "android"))]
pub use local::register_local;

#[cfg(not(target_os = "android"))]
mod local {
    use super::LOCAL_SCHEME;
    use git2::transport::{Service, SmartSubtransport, SmartSubtransportStream, Transport};
    use std::io::{Read, Write};
    use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
    use std::sync::{Arc, Mutex, Once};

    thread_local! {
        /// The device key of the operation running on this thread. libgit2
        /// calls transports on the thread that started the fetch or push.
        pub(super) static KEY: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
        /// What the relay printed on stderr during this thread's operation.
        pub(super) static STDERR: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    }

    /// Registers `teahouse-local://` once per process.
    pub fn register_local() {
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            // SAFETY: registered once, before any remote operation of this
            // engine uses the scheme; the factory is Send + Sync.
            let r = unsafe {
                git2::transport::register(LOCAL_SCHEME, |remote| {
                    Transport::smart(
                        remote,
                        false,
                        Local {
                            conn: Mutex::new(None),
                        },
                    )
                })
            };
            if let Err(e) = r {
                eprintln!("teahouse-client: cannot register {LOCAL_SCHEME}: {e}");
            }
        });
    }

    struct Conn {
        child: Child,
        stdin: ChildStdin,
        stdout: ChildStdout,
        stderr: Option<std::thread::JoinHandle<String>>,
    }

    #[derive(Clone)]
    struct Shared(Arc<Mutex<Conn>>);

    impl Read for Shared {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.0.lock().expect("conn").stdout.read(buf)
        }
    }

    impl Write for Shared {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().expect("conn").stdin.write(buf)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.0.lock().expect("conn").stdin.flush()
        }
    }

    struct Local {
        conn: Mutex<Option<Shared>>,
    }

    fn err(m: impl std::fmt::Display) -> git2::Error {
        git2::Error::from_str(&m.to_string())
    }

    impl SmartSubtransport for Local {
        fn action(
            &self,
            url: &str,
            action: Service,
        ) -> Result<Box<dyn SmartSubtransportStream>, git2::Error> {
            let verb = match action {
                Service::UploadPackLs => "git-upload-pack",
                Service::ReceivePackLs => "git-receive-pack",
                // The second half of a stateful exchange reuses the stream.
                Service::UploadPack | Service::ReceivePack => {
                    let c = self.conn.lock().expect("conn").clone();
                    return c
                        .map(|s| Box::new(s) as Box<dyn SmartSubtransportStream>)
                        .ok_or_else(|| err("no connection to the relay"));
                }
            };
            let rest = url
                .strip_prefix(&format!("{LOCAL_SCHEME}://"))
                .ok_or_else(|| err(format!("not a {LOCAL_SCHEME} URL: {url}")))?;
            let slash = rest
                .find('/')
                .ok_or_else(|| err(format!("bad URL {url}")))?;
            let (house, root) = rest.split_at(slash);
            let key = KEY
                .with(|k| k.borrow().clone())
                .ok_or_else(|| err("no device key for the local transport"))?;
            let bin =
                std::env::var_os("TEAHOUSE_RELAY_BIN").unwrap_or_else(|| "teahouse-relay".into());
            let mut child = Command::new(bin)
                .arg("--root")
                .arg(root)
                .arg("shell")
                .arg(key)
                .env("SSH_ORIGINAL_COMMAND", format!("{verb} '/{house}'"))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| err(format!("cannot run teahouse-relay: {e}")))?;
            // The relay's own refusals (not seated, unknown teahouse) come on
            // stderr, as ssh would show them: pass them on as remote lines.
            let mut stderr = child.stderr.take().expect("piped");
            let stderr = std::thread::spawn(move || {
                let mut s = String::new();
                let _ = stderr.read_to_string(&mut s);
                s
            });
            let conn = Shared(Arc::new(Mutex::new(Conn {
                stdin: child.stdin.take().expect("piped"),
                stdout: child.stdout.take().expect("piped"),
                stderr: Some(stderr),
                child,
            })));
            *self.conn.lock().expect("conn") = Some(conn.clone());
            Ok(Box::new(conn))
        }

        fn close(&self) -> Result<(), git2::Error> {
            if let Some(c) = self.conn.lock().expect("conn").take() {
                if let Ok(mut c) = c.0.lock() {
                    let _ = c.stdin.flush();
                    let _ = c.child.wait();
                    if let Some(s) = c.stderr.take().and_then(|h| h.join().ok()) {
                        STDERR.with(|e| e.borrow_mut().push_str(&s));
                    }
                }
            }
            Ok(())
        }
    }
}

/// Runs `f` with the device key available to the local transport; what the
/// relay printed on stderr is added to `report`.
pub fn with_key<T>(identity: &Identity, report: &Rc<RefCell<Report>>, f: impl FnOnce() -> T) -> T {
    #[cfg(not(target_os = "android"))]
    {
        register_local();
        local::KEY.with(|k| *k.borrow_mut() = Some(identity.public().blob_b64().to_string()));
        local::STDERR.with(|e| e.borrow_mut().clear());
        let out = f();
        local::KEY.with(|k| *k.borrow_mut() = None);
        let err = local::STDERR.with(|e| std::mem::take(&mut *e.borrow_mut()));
        let mut r = report.borrow_mut();
        r.take_text(err.as_bytes());
        r.take_text(b"\n");
        out
    }
    #[cfg(target_os = "android")]
    {
        let _ = (identity, report);
        f()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn house_urls() {
        assert_eq!(
            house_url("122.10.118.71", "dev").unwrap(),
            "ssh://teahouse@122.10.118.71/dev"
        );
        assert_eq!(
            house_url("estel@127.0.0.1:2222", "dev").unwrap(),
            "ssh://estel@127.0.0.1:2222/dev"
        );
        assert_eq!(
            house_url("ssh://teahouse@r.org/", "dev").unwrap(),
            "ssh://teahouse@r.org/dev"
        );
        assert!(house_url("r.org/x", "dev").is_err());
    }
}
