//! Teahouse commands that are not git: invitations and attachments
//! (`invite`, `redeem`, `blob-has`, `blob-put`, `blob-get`, `blob-purge`).
//! They run over the same ssh as git, with the same device key and the same
//! pinned host keys; in development, through the relay's forced command
//! directly, like the `teahouse-local://` git transport.

use crate::error::{Code, Error, Result};
use crate::identity::Identity;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Where a relay is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endpoint {
    Ssh {
        user: String,
        host: String,
        port: u16,
    },
    /// Development: the relay's root on this machine.
    Local { root: PathBuf },
}

impl Endpoint {
    /// From a relay address as people write it (`[user@]host[:port]`,
    /// `ssh://…`, or `teahouse-local:<root>`). The ssh user defaults to
    /// `teahouse` (CLIENT-API.md §4 `invite`).
    pub fn parse(relay: &str) -> Result<Endpoint> {
        let relay = relay.trim();
        if let Some(root) = relay.strip_prefix("teahouse-local:") {
            let root = root.trim_start_matches("//");
            if !root.starts_with('/') {
                return Err(Error::invalid(
                    "teahouse-local needs an absolute relay root",
                ));
            }
            return Ok(Endpoint::Local {
                root: PathBuf::from(root),
            });
        }
        let rest = relay
            .strip_prefix("ssh://")
            .unwrap_or(relay)
            .trim_end_matches('/');
        let rest = rest.split('/').next().unwrap_or(rest);
        let bad = || Error::invalid(format!("not a relay address: {relay:?}")).with("relay", relay);
        let (user, hostport) = match rest.rsplit_once('@') {
            Some((u, h)) => (u.to_string(), h),
            None => ("teahouse".to_string(), rest),
        };
        let (host, port) = match hostport.rsplit_once(':') {
            Some((h, p)) => (h.to_string(), p.parse().map_err(|_| bad())?),
            None => (hostport.to_string(), 22),
        };
        if host.is_empty() || user.is_empty() || host.contains(' ') {
            return Err(bad());
        }
        Ok(Endpoint::Ssh { user, host, port })
    }

    /// From a teahouse's git URL (`ssh://user@host:port/<house>` or
    /// `teahouse-local://<house>/<root>`).
    pub fn of_house_url(url: &str) -> Result<Endpoint> {
        if let Some(rest) = url.strip_prefix("teahouse-local://") {
            let slash = rest
                .find('/')
                .ok_or_else(|| Error::internal(format!("bad URL {url}")))?;
            return Ok(Endpoint::Local {
                root: PathBuf::from(&rest[slash..]),
            });
        }
        let rest = url
            .strip_prefix("ssh://")
            .ok_or_else(|| Error::internal(format!("bad URL {url}")))?;
        Endpoint::parse(rest.split('/').next().unwrap_or(rest))
    }

    /// The address to give people in an invitation (`host[:port]`, with the
    /// user only when it is not `teahouse`).
    pub fn address(&self) -> String {
        match self {
            Endpoint::Local { root } => format!("teahouse-local:{}", root.display()),
            Endpoint::Ssh { user, host, port } => {
                let mut s = String::new();
                if user != "teahouse" {
                    s.push_str(user);
                    s.push('@');
                }
                s.push_str(host);
                if *port != 22 {
                    s.push_str(&format!(":{port}"));
                }
                s
            }
        }
    }
}

/// A word for the relay's command line: single-quoted when needed.
pub fn quote(word: &str) -> String {
    if !word.is_empty()
        && word
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.:@/".contains(&b))
    {
        word.to_string()
    } else {
        format!("'{}'", word.replace('\'', "'\\''"))
    }
}

/// What the relay said on stderr, as reasons (without its program prefix).
fn reasons(stderr: &str) -> Vec<String> {
    stderr
        .lines()
        .map(|l| {
            l.trim()
                .trim_start_matches("teahouse-relay:")
                .trim()
                .to_string()
        })
        .filter(|l| !l.is_empty())
        .collect()
}

fn refused(stderr: &str) -> Error {
    let r = reasons(stderr);
    let mut e = if r.iter().any(|l| l.contains("you have no seat")) {
        Error::new(
            Code::NotAMember,
            "the relay does not know this device / Relay 不认识这台设备",
        )
    } else {
        Error::new(
            Code::RelayRefused,
            r.first()
                .cloned()
                .unwrap_or_else(|| "the relay refused / Relay 拒绝了".into()),
        )
    };
    e.reasons = r;
    e
}

fn offline(e: impl std::fmt::Display) -> Error {
    Error::new(
        Code::Offline,
        format!("cannot reach the relay / 连不上 Relay: {e}"),
    )
}

/// Runs `command` at the relay with `stdin` as its input; its output goes
/// to `stdout`. A non-zero exit is `relay_refused` (or `not_a_member`) with
/// the relay's own words as reasons.
pub fn exec(
    endpoint: &Endpoint,
    identity: &Identity,
    known_hosts: &Path,
    command: &str,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
) -> Result<()> {
    match endpoint {
        Endpoint::Local { root } => exec_local(root, identity, command, stdin, stdout),
        Endpoint::Ssh { user, host, port } => exec_ssh(
            user,
            host,
            *port,
            identity,
            known_hosts,
            command,
            stdin,
            stdout,
        ),
    }
}

/// [`exec`] with no input, returning the output as text.
pub fn exec_text(
    endpoint: &Endpoint,
    identity: &Identity,
    known_hosts: &Path,
    command: &str,
) -> Result<String> {
    let mut out = Vec::new();
    exec(
        endpoint,
        identity,
        known_hosts,
        command,
        &mut std::io::empty(),
        &mut out,
    )?;
    Ok(String::from_utf8_lossy(&out).into_owned())
}

#[cfg(not(target_os = "android"))]
fn exec_local(
    root: &Path,
    identity: &Identity,
    command: &str,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
) -> Result<()> {
    use std::process::{Command, Stdio};
    let bin = std::env::var_os("TEAHOUSE_RELAY_BIN").unwrap_or_else(|| "teahouse-relay".into());
    let mut child = Command::new(bin)
        .arg("--root")
        .arg(root)
        .arg("shell")
        .arg(identity.public().blob_b64())
        .env("SSH_ORIGINAL_COMMAND", command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| offline(format!("cannot run teahouse-relay: {e}")))?;
    let mut input = Vec::new();
    stdin.read_to_end(&mut input)?;
    let mut child_in = child.stdin.take().expect("piped");
    let writer = std::thread::spawn(move || {
        let _ = child_in.write_all(&input);
    });
    let mut err = String::new();
    let mut child_err = child.stderr.take().expect("piped");
    let reader = std::thread::spawn(move || {
        let _ = child_err.read_to_string(&mut err);
        err
    });
    std::io::copy(&mut child.stdout.take().expect("piped"), stdout)?;
    let status = child.wait()?;
    let _ = writer.join();
    let err = reader.join().unwrap_or_default();
    if status.success() {
        Ok(())
    } else {
        Err(refused(&err))
    }
}

#[cfg(target_os = "android")]
fn exec_local(_: &Path, _: &Identity, _: &str, _: &mut dyn Read, _: &mut dyn Write) -> Result<()> {
    Err(Error::invalid(
        "teahouse-local is for development on a computer",
    ))
}

#[allow(clippy::too_many_arguments)]
fn exec_ssh(
    user: &str,
    host: &str,
    port: u16,
    identity: &Identity,
    known_hosts: &Path,
    command: &str,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
) -> Result<()> {
    let addr = (host, port)
        .to_socket_addrs()
        .map_err(offline)?
        .next()
        .ok_or_else(|| offline(format!("{host} has no address")))?;
    let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(15)).map_err(offline)?;
    let mut s = ssh2::Session::new().map_err(offline)?;
    s.set_tcp_stream(tcp);
    s.set_timeout(120_000);
    s.handshake().map_err(offline)?;
    let hash = s
        .host_key_hash(ssh2::HashType::Sha256)
        .ok_or_else(|| offline("no host key"))?;
    let fp: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    crate::transport::pin_host_key(known_hosts, host, &fp)?;
    authenticate(&s, user, identity).map_err(|_| {
        Error::new(
            Code::NotAMember,
            "the relay does not know this device / Relay 不认识这台设备",
        )
    })?;
    let mut ch = s.channel_session().map_err(offline)?;
    ch.exec(command).map_err(offline)?;
    std::io::copy(stdin, &mut ch)?;
    ch.send_eof().map_err(offline)?;
    std::io::copy(&mut ch, stdout)?;
    let mut err = String::new();
    ch.stderr().read_to_string(&mut err)?;
    ch.wait_close().map_err(offline)?;
    let code = ch.exit_status().map_err(offline)?;
    if code == 0 {
        Ok(())
    } else {
        Err(refused(&err))
    }
}

/// The device key, from memory where libssh2 can (its OpenSSL backend),
/// from the key file elsewhere (WinCNG on Windows).
#[cfg(unix)]
fn authenticate(s: &ssh2::Session, user: &str, identity: &Identity) -> Result<()> {
    let private = identity.private_openssh()?;
    s.userauth_pubkey_memory(user, Some(&identity.public_line()), &private, None)
        .map_err(offline)
}

#[cfg(not(unix))]
fn authenticate(s: &ssh2::Session, user: &str, identity: &Identity) -> Result<()> {
    s.userauth_pubkey_file(user, None, identity.private_path(), None)
        .map_err(offline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints() {
        let e = Endpoint::parse("relay.example.org").unwrap();
        assert_eq!(
            e,
            Endpoint::Ssh {
                user: "teahouse".into(),
                host: "relay.example.org".into(),
                port: 22
            }
        );
        assert_eq!(e.address(), "relay.example.org");
        let e = Endpoint::parse("estel@127.0.0.1:2222").unwrap();
        assert_eq!(e.address(), "estel@127.0.0.1:2222");
        assert_eq!(
            Endpoint::parse("ssh://teahouse@r.org:2200/")
                .unwrap()
                .address(),
            "r.org:2200"
        );
        assert_eq!(
            Endpoint::of_house_url("ssh://teahouse@r.org:2200/wutong").unwrap(),
            Endpoint::parse("r.org:2200").unwrap()
        );
        assert_eq!(
            Endpoint::of_house_url("teahouse-local://wutong/tmp/relay").unwrap(),
            Endpoint::Local {
                root: "/tmp/relay".into()
            }
        );
        assert_eq!(
            Endpoint::parse("teahouse-local:/tmp/r").unwrap().address(),
            "teahouse-local:/tmp/r"
        );
        assert!(Endpoint::parse("r.org:http").is_err());
        assert!(Endpoint::parse("").is_err());
    }

    #[test]
    fn quoting() {
        assert_eq!(quote("wutong"), "wutong");
        assert_eq!(quote("王阿姨"), "'王阿姨'");
        assert_eq!(quote("it's"), "'it'\\''s'");
        assert_eq!(quote(""), "''");
        assert_eq!(reasons("teahouse-relay: no\n\n x \n"), ["no", "x"]);
    }
}
