//! teahouse-relay — the role-enforcing git gateway of 茶馆 (Teahouse).
//!
//! Architecture (see docs/RELAY.md):
//! * OpenSSH authenticates members by their ed25519 keys (decades of
//!   battle-testing; we do not write our own crypto or transport).
//! * Every member key is forced into `teahouse-relay shell <key>`, which only
//!   ever execs `git upload-pack` / `git receive-pack` for a teahouse the key
//!   has a seat in.
//! * The shared `pre-receive` hook enforces the seating chart and the
//!   teahouse's laws before any ref moves. Pushes are all-or-nothing.

mod admin;
mod audit;
mod blobs;
mod devices;
mod error;
mod git;
mod hook;
mod invites;
mod root;
mod shell;

use error::{Error, Result};
use root::{Root, DEFAULT_ROOT};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
teahouse-relay — the gatekeeper of 茶馆 (Teahouse)

USAGE:
    teahouse-relay [--root DIR] <command> [args]

    --root DIR   relay data directory (default: $TEAHOUSE_ROOT or /var/lib/teahouse)

OPERATOR COMMANDS:
    init [--binary PATH] [--authorized-keys FILE]
                         create or repair the relay directory and hooks
    open <id> --name NAME --host-id ID --host-name NAME --host-key KEY|@FILE [--signboard TEXT]
                         open a new teahouse with its host (老板)
    found <id> --name NAME [--signboard TEXT] [--days 1-30] [--address HOST[:PORT]]
          [--kind machine [--agent A]]
                         a one-time invitation that opens teahouse <id> with
                         whoever redeems it as host (prints TH-…[@address])
    adopt <id> --from DIR --key KEY|@FILE [--rev REV]
                         继承: re-open a teahouse from the steward's copy
    list                 list teahouses
    members <id>         show a teahouse's seats
    check [<id>]         verify repositories, seating charts and configuration
    authorized-keys [--key BASE64]
                         print OpenSSH authorized_keys lines (AuthorizedKeysCommand)
    sync-keys --file FILE
                         rewrite the teahouse block of an authorized_keys file
    version              print the version

INTERNAL (invoked by sshd / git):
    shell <key>          forced command for a member key
    redeem-shell <key>   forced command for a key with no seat: redeem only
    hook pre-receive|post-receive

DEVELOPMENT ONLY:
    local-ssh <user@host> <command>
                         a GIT_SSH_COMMAND that skips sshd (see docs/RELAY.md)
";

struct Args {
    v: Vec<String>,
}

impl Args {
    fn take_flag(&mut self, name: &str) -> Result<Option<String>> {
        if let Some(i) = self.v.iter().position(|a| a == name) {
            if i + 1 >= self.v.len() {
                return Err(Error(format!("{name} needs a value")));
            }
            let val = self.v.remove(i + 1);
            self.v.remove(i);
            return Ok(Some(val));
        }
        let prefix = format!("{name}=");
        if let Some(i) = self.v.iter().position(|a| a.starts_with(&prefix)) {
            let val = self.v.remove(i)[prefix.len()..].to_string();
            return Ok(Some(val));
        }
        Ok(None)
    }

    fn need_flag(&mut self, name: &str) -> Result<String> {
        self.take_flag(name)?
            .ok_or_else(|| Error(format!("missing {name}")))
    }

    fn positional(&mut self, what: &str) -> Result<String> {
        if self.v.is_empty() {
            return Err(Error(format!("missing {what}")));
        }
        Ok(self.v.remove(0))
    }

    fn finish(self) -> Result<()> {
        if let Some(extra) = self.v.first() {
            return Err(Error(format!("unexpected argument {extra:?}")));
        }
        Ok(())
    }
}

fn run() -> Result<()> {
    let mut args = Args {
        v: std::env::args().skip(1).collect(),
    };
    // The forced command and hooks always pass --root first; keep it simple.
    let root_dir = match args.take_flag("--root")? {
        Some(r) => r,
        None => std::env::var("TEAHOUSE_ROOT").unwrap_or_else(|_| DEFAULT_ROOT.into()),
    };
    let root = Root::new(PathBuf::from(&root_dir));
    if args.v.is_empty() {
        print!("{USAGE}");
        return Ok(());
    }
    let cmd = args.v.remove(0);
    match cmd.as_str() {
        "help" | "--help" | "-h" => {
            print!("{USAGE}");
            Ok(())
        }
        "version" | "--version" | "-V" => {
            println!("teahouse-relay {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        "init" => {
            let binary = args.take_flag("--binary")?;
            let ak = args.take_flag("--authorized-keys")?;
            args.finish()?;
            admin::init(&root, binary.as_deref(), ak.as_deref())
        }
        "open" => {
            let id = args.positional("teahouse id")?;
            let name = args.need_flag("--name")?;
            let signboard = args.take_flag("--signboard")?;
            let host_id = args.need_flag("--host-id")?;
            let host_name = args.need_flag("--host-name")?;
            let host_key = args.need_flag("--host-key")?;
            args.finish()?;
            admin::new_house(
                &root,
                &admin::NewHouse {
                    id: &id,
                    name: &name,
                    signboard: signboard.as_deref(),
                    host_id: &host_id,
                    host_name: &host_name,
                    host_key: &host_key,
                    host_kind: teahouse_core::Kind::Human,
                    host_agent: None,
                },
            )
        }
        "found" => {
            let id = args.positional("teahouse id")?;
            let name = args.need_flag("--name")?;
            let signboard = args.take_flag("--signboard")?;
            let days = args.take_flag("--days")?;
            let address = args.take_flag("--address")?;
            let kind = args.take_flag("--kind")?;
            let agent = args.take_flag("--agent")?;
            args.finish()?;
            let kind = match kind.as_deref() {
                None => teahouse_core::Kind::Human,
                Some(k) => teahouse_core::Kind::parse(k)
                    .ok_or_else(|| Error(format!("unknown kind {k:?} (human or machine)")))?,
            };
            let days = match days {
                None => None,
                Some(d) => Some(
                    d.parse()
                        .map_err(|_| Error(format!("--days {d:?} is not a number")))?,
                ),
            };
            let code = invites::create_founding(
                &root,
                &id,
                &name,
                signboard.as_deref(),
                days,
                kind,
                agent.as_deref(),
            )?;
            match address {
                Some(a) => println!("{code}@{a}"),
                None => println!("{code}"),
            }
            Ok(())
        }
        "adopt" => {
            let id = args.positional("new teahouse id")?;
            let from = args.need_flag("--from")?;
            let key = args.need_flag("--key")?;
            let rev = args.take_flag("--rev")?;
            args.finish()?;
            admin::adopt(
                &root,
                &id,
                std::path::Path::new(&from),
                rev.as_deref(),
                &key,
            )
        }
        "list" => {
            args.finish()?;
            admin::list(&root)
        }
        "members" => {
            let id = args.positional("teahouse id")?;
            args.finish()?;
            admin::show_members(&root, &id)
        }
        "check" => {
            let id = if args.v.is_empty() {
                None
            } else {
                Some(args.positional("id")?)
            };
            args.finish()?;
            admin::check(&root, id.as_deref())
        }
        "authorized-keys" => {
            let only = args.take_flag("--key")?;
            args.finish()?;
            let cfg = root.load_config()?;
            let only = match only {
                Some(k) => match teahouse_core::PublicKey::from_blob_b64(&k) {
                    Ok(k) => Some(k),
                    Err(_) => return Ok(()), // not an ed25519 member key: print nothing
                },
                None => None,
            };
            let lines = admin::authorized_keys_lines(&root, &cfg.binary, only.as_ref())?;
            for l in &lines {
                println!("{l}");
            }
            // A key with no seat anywhere may still redeem an invitation.
            if let (Some(k), true) = (&only, lines.is_empty()) {
                println!("{}", admin::redeem_line(&root, &cfg.binary, k)?);
            }
            Ok(())
        }
        "sync-keys" => {
            let file = args.need_flag("--file")?;
            args.finish()?;
            let cfg = root.load_config()?;
            admin::sync_keys(&root, &cfg.binary, &file)
        }
        "shell" => {
            let key = args.positional("key")?;
            args.finish()?;
            shell::run(&root, &key, false)
        }
        "redeem-shell" => {
            let key = args.positional("key")?;
            args.finish()?;
            shell::run(&root, &key, true)
        }
        "hook" => {
            let which = args.positional("hook name")?;
            args.finish()?;
            match which.as_str() {
                "pre-receive" => hook::pre_receive(&root),
                "post-receive" => hook::post_receive(&root),
                other => Err(Error(format!("unknown hook {other:?}"))),
            }
        }
        "local-ssh" => admin::local_ssh(&root, &args.v),
        other => Err(Error(format!(
            "unknown command {other:?} (try `teahouse-relay help`)"
        ))),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if e.0 != "push refused" {
                eprintln!("teahouse-relay: {e}");
            }
            ExitCode::FAILURE
        }
    }
}
