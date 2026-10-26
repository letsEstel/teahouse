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
use git2::RemoteCallbacks;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub const LOCAL_SCHEME: &str = "teahouse-local";

/// What one fetch or push learned besides its result.
#[derive(Default)]
pub struct Report {
    /// Lines the relay printed (the "remote:" lines of the git program).
    pub remote_lines: Vec<String>,
    /// `(ref, reason)` for every ref the relay refused.
    pub refused: Vec<(String, String)>,
    /// Why the engine's own ssh could not connect or authenticate (the
    /// relay's host key changed, the device key is unknown, offline).
    pub connect_error: Option<Error>,
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

/// Remote callbacks for one operation: the relay's messages collected.
/// (Connecting, the device key and host keys are the `ssh://` transport's:
/// see [`with_key`].)
pub fn callbacks<'a>(report: Rc<RefCell<Report>>) -> RemoteCallbacks<'a> {
    let mut cb = RemoteCallbacks::new();
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

/// The relay's words for "this key has no seat": in this teahouse (law 1),
/// or anywhere (a key that may only redeem an invitation).
pub(crate) fn is_no_seat(line: &str) -> bool {
    line.contains("you have no seat") || line.contains("has no seat yet")
}

/// Maps a failed fetch or push to the API's errors.
pub fn classify(e: &git2::Error, report: &Report) -> Error {
    let reasons = report.reasons();
    if let Some(err) = &report.connect_error {
        let mut err = err.clone();
        err.reasons = reasons;
        return err;
    }
    // Over ssh an unknown key fails authentication; a key seated in another
    // teahouse gets the relay's answer for "not here" (law 1: unknown
    // teahouses and teahouses without your seat look the same).
    let no_seat = reasons.iter().any(|r| is_no_seat(r)) || is_no_seat(e.message());
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
    use super::{CONTEXT, LOCAL_SCHEME, STDERR};
    use git2::transport::{Service, SmartSubtransport, SmartSubtransportStream, Transport};
    use std::io::{Read, Write};
    use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
    use std::sync::{Arc, Mutex, Once};

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
            let key = CONTEXT
                .with(|c| {
                    c.borrow()
                        .as_ref()
                        .map(|(id, _)| id.public().blob_b64().to_string())
                })
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

thread_local! {
    /// The device key and the pinned host keys of the fetch or push running
    /// on this thread: libgit2 calls transports on the thread that started
    /// it.
    static CONTEXT: RefCell<Option<(Identity, PathBuf)>> = const { RefCell::new(None) };
    /// What the relay printed on stderr during this thread's operation.
    static STDERR: RefCell<String> = const { RefCell::new(String::new()) };
    /// Why this thread's operation could not connect.
    static CONNECT_ERROR: RefCell<Option<Error>> = const { RefCell::new(None) };
}

/// Runs `f` (a fetch or push) with the device key and the pinned host keys
/// available to the transports; what the relay printed on stderr, and why
/// a connection failed, are added to `report`.
pub fn with_key<T>(
    identity: &Identity,
    known_hosts: &Path,
    report: &Rc<RefCell<Report>>,
    f: impl FnOnce() -> T,
) -> T {
    ssh::register();
    #[cfg(not(target_os = "android"))]
    register_local();
    CONTEXT.with(|c| *c.borrow_mut() = Some((identity.clone(), known_hosts.to_path_buf())));
    STDERR.with(|e| e.borrow_mut().clear());
    CONNECT_ERROR.with(|e| *e.borrow_mut() = None);
    let out = f();
    CONTEXT.with(|c| *c.borrow_mut() = None);
    let err = STDERR.with(|e| std::mem::take(&mut *e.borrow_mut()));
    let mut r = report.borrow_mut();
    r.take_text(err.as_bytes());
    r.take_text(b"\n");
    r.connect_error = CONNECT_ERROR.with(|e| e.borrow_mut().take());
    out
}

// ---------------------------------------------------------------------------
// The ssh transport

/// `ssh://` through the engine's own ssh ([`crate::remote::connect`]), in
/// place of libgit2's: one ssh for git and for the other relay commands,
/// with one way of pinning the relay's host key.
mod ssh {
    use super::{CONNECT_ERROR, CONTEXT, STDERR};
    use crate::remote::{self, Endpoint};
    use git2::transport::{Service, SmartSubtransport, SmartSubtransportStream, Transport};
    use std::io::{Read, Write};
    use std::sync::{Arc, Mutex, Once};

    /// Registers `ssh://` once per process.
    pub fn register() {
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            // SAFETY: registered once, before any remote operation of this
            // engine uses the scheme; the factory is Send + Sync.
            let r = unsafe {
                git2::transport::register("ssh", |remote| {
                    Transport::smart(
                        remote,
                        false,
                        Ssh {
                            conn: Mutex::new(None),
                        },
                    )
                })
            };
            if let Err(e) = r {
                eprintln!("teahouse-client: cannot register ssh: {e}");
            }
        });
    }

    struct Conn {
        // Kept alive as long as the channel.
        _session: ssh2::Session,
        channel: ssh2::Channel,
    }

    #[derive(Clone)]
    struct Shared(Arc<Mutex<Conn>>);

    impl Conn {
        /// The relay's own refusals (not seated, unknown teahouse) come on
        /// stderr: pass them on as remote lines. Taken as soon as the relay
        /// is done: on a refusal libgit2 returns before it closes us.
        fn take_stderr(&mut self) {
            let mut s = String::new();
            let _ = self.channel.stderr().read_to_string(&mut s);
            STDERR.with(|e| e.borrow_mut().push_str(&s));
        }
    }

    impl Read for Shared {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let mut c = self.0.lock().expect("conn");
            let n = c.channel.read(buf)?;
            if n == 0 {
                c.take_stderr();
            }
            Ok(n)
        }
    }

    impl Write for Shared {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let mut c = self.0.lock().expect("conn");
            let r = c.channel.write(buf);
            if r.is_err() {
                c.take_stderr();
            }
            r
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.0.lock().expect("conn").channel.flush()
        }
    }

    struct Ssh {
        conn: Mutex<Option<Shared>>,
    }

    fn err(m: impl std::fmt::Display) -> git2::Error {
        git2::Error::from_str(&m.to_string())
    }

    impl SmartSubtransport for Ssh {
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
            let Ok(Endpoint::Ssh { user, host, port }) = Endpoint::of_house_url(url) else {
                return Err(err(format!("not a relay URL: {url}")));
            };
            let house = url.trim_end_matches('/').rsplit('/').next().unwrap_or("");
            let (identity, known_hosts) = CONTEXT
                .with(|c| c.borrow().clone())
                .ok_or_else(|| err("no device key for the ssh transport"))?;
            let fail = |e: crate::error::Error| {
                let m = err(&e.message);
                CONNECT_ERROR.with(|c| *c.borrow_mut() = Some(e));
                m
            };
            let session =
                remote::connect(&user, &host, port, &identity, &known_hosts).map_err(fail)?;
            let mut channel = session.channel_session().map_err(err)?;
            channel.exec(&format!("{verb} '/{house}'")).map_err(err)?;
            let conn = Shared(Arc::new(Mutex::new(Conn {
                _session: session,
                channel,
            })));
            *self.conn.lock().expect("conn") = Some(conn.clone());
            Ok(Box::new(conn))
        }

        fn close(&self) -> Result<(), git2::Error> {
            if let Some(c) = self.conn.lock().expect("conn").take() {
                if let Ok(mut c) = c.0.lock() {
                    let _ = c.channel.send_eof();
                    c.take_stderr();
                    let _ = c.channel.wait_close();
                }
            }
            Ok(())
        }
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
