//! The forced command OpenSSH runs for every member key.
//!
//! OpenSSH has already proven the client holds the private key; sshd then
//! runs `teahouse-relay shell <key>` with the client's request in
//! `SSH_ORIGINAL_COMMAND`. This module decides whether that key may read or
//! write the requested teahouse and, if so, replaces itself with the real
//! `git upload-pack` / `git receive-pack`. Besides git, it serves a short,
//! fixed list of teahouse commands (attachments, invitations); nothing else
//! can be executed.
//!
//! Keys with no seat anywhere get `redeem-shell` instead (INVITE.md rule 6):
//! the same entry, restricted to `redeem`.

use crate::audit::{self, Entry};
use crate::bail;
use crate::error::Result;
use crate::root::{valid_house_id, Root};
use crate::{blobs, devices, invites};
use std::os::unix::process::CommandExt;
use std::process::Command;
use teahouse_core::members::{Kind, Member, Role};
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

/// Splits a command line into words: spaces separate, single quotes
/// quote (POSIX: `'it'\''s'`). Nothing is expanded.
pub fn split_words(cmd: &str) -> Result<Vec<String>> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut in_word = false;
    let mut chars = cmd.chars();
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                in_word = true;
                loop {
                    match chars.next() {
                        Some('\'') => break,
                        Some(c) => cur.push(c),
                        None => bail!("unterminated quote"),
                    }
                }
            }
            ' ' | '\t' | '\n' => {
                if in_word {
                    words.push(std::mem::take(&mut cur));
                    in_word = false;
                }
            }
            '\\' | '"' | '$' | '`' | ';' | '|' | '&' | '<' | '>' => {
                bail!("unsupported character {c:?} in command")
            }
            c => {
                in_word = true;
                cur.push(c);
            }
        }
    }
    if in_word {
        words.push(cur);
    }
    Ok(words)
}

/// `--flag value` options after the positional words.
struct Opts {
    words: Vec<String>,
}

impl Opts {
    fn flag(&mut self, name: &str) -> Result<Option<String>> {
        match self.words.iter().position(|w| w == name) {
            None => Ok(None),
            Some(i) if i + 1 < self.words.len() => {
                let v = self.words.remove(i + 1);
                self.words.remove(i);
                Ok(Some(v))
            }
            Some(_) => bail!("{name} needs a value"),
        }
    }

    fn positional(&mut self, what: &str) -> Result<String> {
        if self.words.is_empty() {
            bail!("missing {what}");
        }
        Ok(self.words.remove(0))
    }

    fn done(self) -> Result<()> {
        match self.words.first() {
            Some(w) => bail!("unexpected argument {w:?}"),
            None => Ok(()),
        }
    }
}

/// The house and the seat of `key` in it, or the one answer for both
/// "no such teahouse" and "no seat here" (law 1).
fn seated(root: &Root, id: &str, key: &PublicKey) -> Result<(crate::root::House, Member)> {
    let denied = |reason: &str| {
        audit::log(
            root,
            Entry::new("shell.denied")
                .house(id)
                .key(&key.short())
                .field("reason", reason.to_string()),
        );
    };
    let Ok(house) = root.house(id) else {
        denied("no such house");
        bail!("{NOT_FOUND}");
    };
    let members = house.members()?;
    let Some(member) = members.by_key(key).cloned() else {
        denied("not a member");
        bail!("{NOT_FOUND}");
    };
    Ok((house, member))
}

fn host_only(member: &Member, what: &str) -> Result<()> {
    if member.role != Role::Host {
        bail!(
            "only the host can {what} / 只有老板能{}",
            if what == "purge attachments" {
                "删除附件"
            } else {
                "管理邀请"
            }
        );
    }
    Ok(())
}

/// The teahouse commands besides git.
fn command(root: &Root, key: &PublicKey, words: Vec<String>, restricted: bool) -> Result<()> {
    let mut o = Opts { words };
    let verb = o.positional("command")?;
    if restricted && verb != "redeem" {
        audit::log(
            root,
            Entry::new("shell.refused")
                .key(&key.short())
                .field("command", verb.clone())
                .field("reason", "no seat"),
        );
        bail!("this key has no seat yet; it can only redeem an invitation / 这把钥匙还没有座位，只能兑换邀请");
    }
    match verb.as_str() {
        "redeem" => {
            let code = o.positional("invitation code")?;
            let name = o
                .flag("--name")?
                .ok_or_else(|| crate::error::Error("missing --name".into()))?;
            let agent = o.flag("--agent")?;
            o.done()?;
            let s = invites::redeem(root, key, &code, &name, agent.as_deref())?;
            println!("house {}", s.house);
            println!("member {}", s.member);
            // A pairing code adds the device in every teahouse at once.
            for (house, member) in &s.more {
                println!("house {house}");
                println!("member {member}");
            }
            Ok(())
        }
        "pair" => {
            let name = o.flag("--name")?;
            let minutes = o.flag("--minutes")?;
            o.done()?;
            let minutes = match minutes {
                None => None,
                Some(m) => Some(m.parse().map_err(|_| {
                    crate::error::Error(format!("--minutes {m:?} is not a number"))
                })?),
            };
            println!("{}", devices::create(root, key, name.as_deref(), minutes)?);
            Ok(())
        }
        "unpair" => {
            let house = o.positional("teahouse")?;
            let which = o.positional("device name or SHA256: fingerprint")?;
            o.done()?;
            seated(root, &house, key)?;
            println!("{}", devices::unpair(root, key, &house, &which)?);
            Ok(())
        }
        "invite" => {
            let house = o.positional("teahouse")?;
            let role = o.flag("--role")?;
            let name = o.flag("--name")?;
            let days = o.flag("--days")?;
            let kind = o.flag("--kind")?;
            let agent = o.flag("--agent")?;
            o.done()?;
            let (_, me) = seated(root, &house, key)?;
            host_only(&me, "invite")?;
            let kind = match kind.as_deref() {
                None => Kind::Human,
                Some(k) => Kind::parse(k).ok_or_else(|| {
                    crate::error::Error(format!("unknown kind {k:?} (human or machine)"))
                })?,
            };
            let role = match role.as_deref() {
                None => Role::Guest,
                Some(r) => Role::parse(r)
                    .ok_or_else(|| crate::error::Error(format!("unknown role {r:?}")))?,
            };
            let days =
                match days {
                    None => None,
                    Some(d) => Some(d.parse().map_err(|_| {
                        crate::error::Error(format!("--days {d:?} is not a number"))
                    })?),
                };
            println!(
                "{}",
                invites::create(
                    root,
                    &house,
                    &me,
                    &invites::Seat {
                        role,
                        kind,
                        name: name.as_deref(),
                        agent: agent.as_deref(),
                    },
                    days
                )?
            );
            Ok(())
        }
        "uninvite" => {
            let house = o.positional("teahouse")?;
            let prefix = o.positional("first four characters of the code")?;
            o.done()?;
            let (_, me) = seated(root, &house, key)?;
            host_only(&me, "invite")?;
            invites::revoke(root, &house, &me, &prefix)
        }
        "blob-has" | "blob-get" | "blob-purge" => {
            let house = o.positional("teahouse")?;
            let hash = o.positional("sha256")?;
            o.done()?;
            let (_, me) = seated(root, &house, key)?;
            match verb.as_str() {
                "blob-has" => {
                    println!(
                        "{}",
                        if blobs::has(root, &house, &hash)? {
                            "yes"
                        } else {
                            "no"
                        }
                    );
                    Ok(())
                }
                "blob-get" => blobs::get(root, &house, &hash, &mut std::io::stdout().lock()),
                _ => {
                    host_only(&me, "purge attachments")?;
                    blobs::purge(root, &house, &me.id, &hash)
                }
            }
        }
        "blob-put" => {
            let house = o.positional("teahouse")?;
            let hash = o.positional("sha256")?;
            let size = o.positional("size")?;
            o.done()?;
            let size: u64 = size
                .parse()
                .map_err(|_| crate::error::Error(format!("size {size:?} is not a number")))?;
            let (_, me) = seated(root, &house, key)?;
            let quota = root.load_config()?.blob_quota;
            blobs::put(
                root,
                &house,
                &me.id,
                &hash,
                size,
                &mut std::io::stdin().lock(),
                quota,
            )
        }
        other => {
            audit::log(
                root,
                Entry::new("shell.refused")
                    .key(&key.short())
                    .field("command", other.to_string())
                    .field("reason", "unknown command"),
            );
            bail!("only git fetch/clone/push are served here / 这里只提供 git 读写")
        }
    }
}

pub fn run(root: &Root, key_b64: &str, restricted: bool) -> Result<()> {
    let key = PublicKey::from_blob_b64(key_b64)?;
    let original = std::env::var("SSH_ORIGINAL_COMMAND").unwrap_or_default();

    if !restricted && (original.trim().is_empty() || original.trim() == "info") {
        return info(root, &key);
    }
    let trimmed = original.trim_start();
    if !trimmed.starts_with("git") || restricted {
        let words = match split_words(&original) {
            Ok(w) if !w.is_empty() => w,
            Ok(_) => vec!["redeem".to_string()],
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
        return command(root, &key, words, restricted);
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
    let mut ids = Vec::new();
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
            ids.push(id);
        }
    }
    audit::log(
        root,
        Entry::new("shell.info")
            .key(&key.short())
            .field("houses", ids.join(",")),
    );
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
