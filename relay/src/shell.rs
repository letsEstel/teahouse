//! The forced command OpenSSH runs for every member key.
//!
//! OpenSSH has already proven the client holds the private key; sshd then
//! runs `teahouse-relay shell <key>` with the client's request in
//! `SSH_ORIGINAL_COMMAND`. This module decides whether that key may read or
//! write the requested teahouse and, if so, replaces itself with the real
//! `git upload-pack` / `git receive-pack`. Nothing else can be executed.

use crate::audit::{self, Entry};
use crate::bail;
use crate::error::Result;
use crate::root::{valid_house_id, Root};
use std::os::unix::process::CommandExt;
use std::process::Command;
use teahouse_core::PublicKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    Read,
    Write,
}

impl Service {
    fn git_subcommand(self) -> &'static str {
        match self {
            Service::Read => "upload-pack",
            Service::Write => "receive-pack",
        }
    }
}

/// Parses `git-upload-pack '/wutong.git'` and friends.
/// Accepts `git-upload-pack` / `git upload-pack` (and receive-pack), a path
/// that is optionally single-quoted, optionally prefixed with `/` or `~/`,
/// and optionally suffixed with `.git`. Everything else is refused.
pub fn parse_request(cmd: &str) -> Result<(Service, String)> {
    let cmd = cmd.trim();
    let (verb, rest) = if let Some(r) = cmd.strip_prefix("git-upload-pack ") {
        (Service::Read, r)
    } else if let Some(r) = cmd.strip_prefix("git upload-pack ") {
        (Service::Read, r)
    } else if let Some(r) = cmd.strip_prefix("git-receive-pack ") {
        (Service::Write, r)
    } else if let Some(r) = cmd.strip_prefix("git receive-pack ") {
        (Service::Write, r)
    } else {
        bail!("only git fetch/clone/push are served here / 这里只提供 git 读写");
    };
    let rest = rest.trim();
    let path = match rest.strip_prefix('\'') {
        Some(inner) => match inner.strip_suffix('\'') {
            Some(p) if !p.contains('\'') => p,
            _ => bail!("malformed repository path"),
        },
        None => rest,
    };
    let path = path
        .strip_prefix("~/")
        .or_else(|| path.strip_prefix('/'))
        .unwrap_or(path);
    let path = path.strip_suffix('/').unwrap_or(path);
    let id = path.strip_suffix(".git").unwrap_or(path);
    if !valid_house_id(id) {
        bail!("no such teahouse / 没有这间茶馆");
    }
    Ok((verb, id.to_string()))
}

const NOT_FOUND: &str =
    "teahouse not found, or you have no seat in it / 茶馆不存在，或者你在这里没有座位";

pub fn run(root: &Root, key_b64: &str) -> Result<()> {
    let key = PublicKey::from_blob_b64(key_b64)?;
    let original = std::env::var("SSH_ORIGINAL_COMMAND").unwrap_or_default();

    if original.trim().is_empty() || original.trim() == "info" {
        return info(root, &key);
    }

    let (service, id) = match parse_request(&original) {
        Ok(v) => v,
        Err(e) => {
            audit::log(
                root,
                Entry::new("shell.refused")
                    .key(&key.short())
                    .field("command", original.clone())
                    .field("reason", e.0.clone()),
            );
            return Err(e);
        }
    };

    let house = match root.house(&id) {
        Ok(h) => h,
        Err(_) => {
            audit::log(
                root,
                Entry::new("shell.denied")
                    .house(&id)
                    .key(&key.short())
                    .field("reason", "no such house"),
            );
            bail!("{NOT_FOUND}");
        }
    };
    let members = house.members()?;
    let Some(member) = members.by_key(&key) else {
        audit::log(
            root,
            Entry::new("shell.denied")
                .house(&id)
                .key(&key.short())
                .field("reason", "not a member"),
        );
        bail!("{NOT_FOUND}");
    };

    audit::log(
        root,
        Entry::new(match service {
            Service::Read => "shell.read",
            Service::Write => "shell.write",
        })
        .house(&id)
        .member(&member.id)
        .key(&key.short())
        .field("role", member.role.as_str()),
    );

    let root_str = root.dir_str()?;
    let mut cmd = Command::new("git");
    cmd.arg(service.git_subcommand());
    if service == Service::Read {
        cmd.arg("--strict"); // never fall back to <path>/.git
    }
    cmd.arg(&house.path);
    for v in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_NAMESPACE",
        "GIT_CONFIG",
        "GIT_CONFIG_PARAMETERS",
        "GIT_CONFIG_COUNT",
        "TEAHOUSE_KEY",
        "TEAHOUSE_HOUSE",
        "TEAHOUSE_ROOT",
    ] {
        cmd.env_remove(v);
    }
    cmd.env("GIT_CONFIG_NOSYSTEM", "1");
    cmd.env("TEAHOUSE_KEY", key.blob_b64());
    cmd.env("TEAHOUSE_HOUSE", &id);
    cmd.env("TEAHOUSE_ROOT", &root_str);
    let err = cmd.exec();
    bail!("cannot start git {}: {err}", service.git_subcommand())
}

fn info(root: &Root, key: &PublicKey) -> Result<()> {
    let mut seats = Vec::new();
    for id in root.house_ids()? {
        let Ok(h) = root.house(&id) else { continue };
        let Ok(m) = h.members() else { continue };
        if let Some(me) = m.by_key(key) {
            seats.push(format!(
                "  {id:<24} {}  {} ({})",
                m.name,
                me.role.zh(),
                me.role
            ));
        }
    }
    if seats.is_empty() {
        println!("你在这个 relay 上还没有座位。 You have no seat on this relay yet.");
    } else {
        println!("你的茶馆 · Your teahouses:");
        for s in seats {
            println!("{s}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests() {
        for (cmd, s, id) in [
            ("git-upload-pack '/wutong.git'", Service::Read, "wutong"),
            ("git-upload-pack 'wutong'", Service::Read, "wutong"),
            ("git upload-pack '~/wutong.git'", Service::Read, "wutong"),
            ("git-receive-pack '/kant-club'", Service::Write, "kant-club"),
            ("git-receive-pack wutong.git", Service::Write, "wutong"),
            ("git-upload-pack '/wutong.git/'", Service::Read, "wutong"),
        ] {
            assert_eq!(parse_request(cmd).unwrap(), (s, id.to_string()), "{cmd}");
        }
        for bad in [
            "bash",
            "git-upload-archive '/wutong'",
            "git-upload-pack '/../etc'",
            "git-upload-pack '/a/b'",
            "git-upload-pack 'x'; rm -rf /",
            "git-upload-pack '/x'y'",
            "git-receive-pack '/Wutong'",
            "git-upload-pack ",
            "scp -t /tmp",
        ] {
            assert!(parse_request(bad).is_err(), "{bad}");
        }
    }
}
