//! Operator commands: setting up a relay, opening teahouses, succession,
//! keys for OpenSSH, and health checks. These run as the relay's own user,
//! write repositories directly with git plumbing, and never go through hooks.

use crate::audit::{self, Entry};
use crate::bail;
use crate::error::{Context, Result};
use crate::git::{commit_files, Git, Signature};
use crate::root::{members_at, safe_path, valid_house_id, write_atomic, Root};
use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use teahouse_core::members::{valid_member_id, Member, Members, Role, MEMBERS_PATH};
use teahouse_core::rules::MAIN_REF;
use teahouse_core::PublicKey;

/// Repository settings every teahouse gets. Defence in depth: the hook is the
/// authority, but git itself also refuses what should never happen.
fn house_config(root: &Root) -> Result<Vec<(&'static str, String)>> {
    Ok(vec![
        (
            "core.hooksPath",
            root.hooks_dir().to_string_lossy().into_owned(),
        ),
        ("core.logAllRefUpdates", "always".into()),
        ("receive.denyNonFastForwards", "true".into()),
        ("receive.fsckObjects", "true".into()),
        ("transfer.fsckObjects", "true".into()),
        ("receive.maxInputSize", "209715200".into()),
        ("receive.advertisePushOptions", "false".into()),
        ("uploadpack.allowFilter", "true".into()),
        ("gc.reflogExpire", "never".into()),
        ("gc.reflogExpireUnreachable", "never".into()),
    ])
}

pub fn init(root: &Root, binary: Option<&str>, authorized_keys: Option<&str>) -> Result<()> {
    let root_s = root.dir_str()?;
    let bin = match binary {
        Some(b) => b.to_string(),
        None => std::env::current_exe()?
            .canonicalize()?
            .to_str()
            .map(str::to_string)
            .unwrap_or_default(),
    };
    if !safe_path(&bin) {
        bail!("relay binary path {bin:?} must be absolute and use only [A-Za-z0-9/._-+@]; pass --binary");
    }
    if let Some(a) = authorized_keys {
        if !safe_path(a) {
            bail!("authorized_keys path {a:?} must be a safe absolute path");
        }
    }
    for d in [
        root.dir.clone(),
        root.houses_dir(),
        root.hooks_dir(),
        root.dir.join("log"),
        root.tmp_dir(),
    ] {
        fs::create_dir_all(&d).context(|| format!("cannot create {}", d.display()))?;
    }
    fs::set_permissions(&root.dir, fs::Permissions::from_mode(0o750))?;

    let mut conf =
        String::from("# teahouse-relay configuration (written by `teahouse-relay init`)\n");
    conf.push_str(&format!("binary = {bin}\n"));
    if let Some(a) = authorized_keys {
        conf.push_str(&format!("authorized_keys = {a}\n"));
    }
    write_atomic(&root.conf_path(), conf.as_bytes(), 0o640)?;

    for hook in ["pre-receive", "post-receive"] {
        let script = format!(
            "#!/bin/sh\n# Installed by teahouse-relay. Do not edit; re-run `teahouse-relay init`.\nexec '{bin}' --root '{root_s}' hook {hook}\n"
        );
        write_atomic(&root.hooks_dir().join(hook), script.as_bytes(), 0o755)?;
    }
    // Existing houses pick up the (possibly moved) hooks directory.
    for id in root.house_ids()? {
        configure_house(root, &Git::at(root.house_path(&id)))?;
    }
    // The relay signs the seat commits it writes for invitations.
    if let Err(e) = crate::invites::ensure_signing_key(root) {
        eprintln!("teahouse-relay: warning: {e}; invitations will not work until it exists");
    }
    audit::log(root, Entry::new("relay.init").field("binary", bin.clone()));
    println!("relay ready at {root_s}");
    println!("  binary: {bin}");
    Ok(())
}

fn configure_house(root: &Root, git: &Git) -> Result<()> {
    for (k, v) in house_config(root)? {
        git.run(&["config", k, &v])?;
    }
    Ok(())
}

fn create_bare(root: &Root, id: &str) -> Result<(Git, std::path::PathBuf)> {
    if !valid_house_id(id) {
        bail!("teahouse id {id:?} must be lowercase letters, digits and '-' (max 63)");
    }
    root.load_config()?; // refuse to work on an uninitialised root
    let path = root.house_path(id);
    if path.exists() {
        bail!("teahouse {id:?} already exists");
    }
    let tmp = root
        .tmp_dir()
        .join(format!("new-{id}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    let out = std::process::Command::new("git")
        .args(["init", "--bare", "--quiet", "--initial-branch=main"])
        .arg(&tmp)
        .env_remove("GIT_DIR")
        .output()?;
    if !out.status.success() {
        bail!(
            "git init failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let git = Git::at(&tmp);
    configure_house(root, &git)?;
    git.run(&["config", "teahouse.id", id])?;
    Ok((git, tmp))
}

/// Moves a fully prepared repository into place in one rename, so a teahouse
/// is either absent or complete — never half-created.
fn publish(root: &Root, from: &Path, id: &str) -> Result<()> {
    let to = root.house_path(id);
    fs::rename(from, &to).context(|| format!("cannot move teahouse into {}", to.display()))?;
    Ok(())
}

pub struct NewHouse<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub signboard: Option<&'a str>,
    pub host_id: &'a str,
    pub host_name: &'a str,
    pub host_key: &'a str,
}

pub fn read_key_arg(arg: &str) -> Result<PublicKey> {
    let text = match arg.strip_prefix('@') {
        Some(path) => {
            fs::read_to_string(path).context(|| format!("cannot read key file {path}"))?
        }
        None => arg.to_string(),
    };
    Ok(PublicKey::parse(text.trim())?)
}

pub fn new_house(root: &Root, h: &NewHouse<'_>) -> Result<()> {
    if !valid_member_id(h.host_id) {
        bail!(
            "host id {:?} must be lowercase letters, digits, '-' or '_' (max 32)",
            h.host_id
        );
    }
    let key = read_key_arg(h.host_key)?;
    let members = Members {
        name: h.name.to_string(),
        signboard: h.signboard.map(str::to_string),
        members: vec![Member {
            id: h.host_id.into(),
            name: h.host_name.into(),
            role: Role::Host,
            key,
        }],
    };
    let toml = members.to_toml()?;

    let mut readme = format!("# {}\n\n", h.name);
    if let Some(s) = h.signboard {
        readme.push_str(&format!("> {s}\n\n"));
    }
    readme.push_str("这是一间茶馆。问题写在 `problems/`，成果放在 `works/`，公告贴在 `board/`。\n");
    readme.push_str(
        "This is a teahouse: problems in `problems/`, results in `works/`, notices in `board/`.\n",
    );

    let (git, staged) = create_bare(root, h.id)?;
    let res = (|| {
        let message = format!(
            "开张：{}\n\nOpened by {} ({}).\n\nProblem: meta\n",
            h.name, h.host_name, h.host_id
        );
        let commit = commit_files(
            &git,
            &root.tmp_dir(),
            None,
            &[
                (MEMBERS_PATH, toml.as_bytes()),
                ("README.md", readme.as_bytes()),
            ],
            &message,
            &Signature {
                name: h.host_name,
                email: &format!("{}@teahouse.invalid", h.host_id),
            },
        )?;
        git.run(&[
            "update-ref",
            "-m",
            "teahouse: opened",
            MAIN_REF,
            &commit,
            "0000000000000000000000000000000000000000",
        ])?;
        git.run(&["symbolic-ref", "HEAD", MAIN_REF])?;
        publish(root, &staged, h.id)?;
        Ok::<_, crate::error::Error>(commit)
    })();
    let commit = match res {
        Ok(c) => c,
        Err(e) => {
            let _ = fs::remove_dir_all(&staged);
            return Err(e);
        }
    };
    audit::log(
        root,
        Entry::new("house.opened")
            .house(h.id)
            .member(h.host_id)
            .field("commit", commit.clone()),
    );
    println!(
        "opened teahouse {:?} ({}) — host {} — main at {}",
        h.id,
        h.name,
        h.host_id,
        &commit[..10]
    );
    Ok(())
}

/// 继承: the steward re-opens a teahouse on this relay from their own copy.
///
/// The history is imported unchanged (every commit keeps its hash), then one
/// succession commit promotes the steward to host and gives the previous host
/// a guest seat. Only the key listed as steward on that history may do this.
pub fn adopt(root: &Root, id: &str, from: &Path, rev: Option<&str>, key_arg: &str) -> Result<()> {
    let key = read_key_arg(key_arg)?;
    let src_dir = {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(from)
            .args(["rev-parse", "--absolute-git-dir"])
            .env_remove("GIT_DIR")
            .output()?;
        if !out.status.success() {
            bail!("{} is not a git repository", from.display());
        }
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    let src = Git::at(&src_dir);
    let candidates: Vec<&str> = match rev {
        Some(r) => vec![r],
        None => vec!["refs/remotes/origin/main", "refs/heads/main"],
    };
    let mut tip = None;
    for c in &candidates {
        if let Some(oid) = src.resolve(c)? {
            tip = Some((c.to_string(), oid));
            break;
        }
    }
    let Some((tip_ref, tip)) = tip else {
        bail!(
            "found none of {} in {}",
            candidates.join(", "),
            from.display()
        );
    };

    let members = members_at(&src, &tip)?;
    let Some(me) = members.by_key(&key) else {
        bail!("that key has no seat in this teahouse");
    };
    if me.role != Role::Steward {
        bail!(
            "only the steward (伙计) can re-open a teahouse; {} is {}",
            me.id,
            me.role
        );
    }
    let old_host = members.host().id.clone();
    let next = members.succeed()?;
    let toml = next.to_toml()?;

    let (git, staged) = create_bare(root, id)?;
    let res = (|| {
        let refspec = format!("{tip}:refs/heads/main");
        git.run(&["fetch", "--quiet", "--no-tags", &src_dir, &refspec])?;
        git.run(&["fsck", "--no-progress", "--no-dangling"])?;
        let message = format!(
            "继承：{} 接管茶馆\n\n{} ({}) re-opened this teahouse from their copy at {}.\nThe previous host {} keeps a guest seat.\n\nProblem: meta\nSuccession-From: {}\n",
            me.name, me.name, me.id, &tip[..10], old_host, old_host
        );
        let commit = commit_files(
            &git,
            &root.tmp_dir(),
            Some(&tip),
            &[(MEMBERS_PATH, toml.as_bytes())],
            &message,
            &Signature {
                name: &me.name,
                email: &format!("{}@teahouse.invalid", me.id),
            },
        )?;
        git.run(&[
            "update-ref",
            "-m",
            "teahouse: succession",
            MAIN_REF,
            &commit,
            &tip,
        ])?;
        git.run(&["symbolic-ref", "HEAD", MAIN_REF])?;
        publish(root, &staged, id)?;
        Ok::<_, crate::error::Error>(commit)
    })();
    let commit = match res {
        Ok(c) => c,
        Err(e) => {
            let _ = fs::remove_dir_all(&staged);
            return Err(e);
        }
    };
    audit::log(
        root,
        Entry::new("house.adopted")
            .house(id)
            .member(&me.id)
            .field("from", tip_ref)
            .field("previous_host", old_host.clone())
            .field("commit", commit.clone()),
    );
    println!(
        "teahouse {id:?} re-opened by {} — history kept up to {} — succession commit {}",
        me.id,
        &tip[..10],
        &commit[..10]
    );
    println!("previous host {old_host} now has a guest seat");
    Ok(())
}

pub fn list(root: &Root) -> Result<()> {
    let ids = root.house_ids()?;
    if ids.is_empty() {
        println!("(no teahouses yet)");
    }
    for id in ids {
        match root.house(&id).and_then(|h| h.members()) {
            Ok(m) => println!(
                "{id:<24} {:>2}/15  host {:<12} {}",
                m.members.len(),
                m.host().id,
                m.name
            ),
            Err(e) => println!("{id:<24} BROKEN: {e}"),
        }
    }
    Ok(())
}

pub fn show_members(root: &Root, id: &str) -> Result<()> {
    let m = root.house(id)?.members()?;
    println!("{} ({} / 15)", m.name, m.members.len());
    if let Some(s) = &m.signboard {
        println!("  「{s}」");
    }
    for mem in &m.members {
        println!(
            "  {:<8} {:<16} {:<12} {}",
            mem.role.as_str(),
            mem.id,
            mem.name,
            mem.key.short()
        );
    }
    Ok(())
}

/// Health check: repository integrity, seating chart, and configuration.
pub fn check(root: &Root, only: Option<&str>) -> Result<()> {
    let cfg = root.load_config()?;
    if !Path::new(&cfg.binary).exists() {
        bail!(
            "configured binary {} does not exist; re-run `teahouse-relay init`",
            cfg.binary
        );
    }
    let ids = match only {
        Some(id) => vec![id.to_string()],
        None => root.house_ids()?,
    };
    let hooks = root.hooks_dir().to_string_lossy().into_owned();
    let mut bad = 0;
    for id in &ids {
        let result = (|| {
            let h = root.house(id)?;
            h.git.run(&["fsck", "--no-progress", "--no-dangling"])?;
            let m = h.members()?;
            let hp = h
                .git
                .run_str(&["config", "core.hooksPath"])
                .unwrap_or_default();
            if hp != hooks {
                bail!("core.hooksPath is {hp:?}, expected {hooks:?} (run `teahouse-relay init`)");
            }
            Ok::<_, crate::error::Error>(m)
        })();
        match result {
            Ok(m) => println!(
                "ok    {id:<24} {} seats, host {}",
                m.members.len(),
                m.host().id
            ),
            Err(e) => {
                bad += 1;
                println!("FAIL  {id:<24} {e}");
            }
        }
    }
    if bad > 0 {
        bail!("{bad} of {} teahouses failed the check", ids.len());
    }
    Ok(())
}

/// The line for a key with no seat anywhere (AuthorizedKeysCommand only):
/// it may redeem an invitation and nothing else (INVITE.md rule 6).
pub fn redeem_line(root: &Root, binary: &str, key: &PublicKey) -> Result<String> {
    let root_s = root.dir_str()?;
    Ok(format!(
        "restrict,command=\"{binary} --root {root_s} redeem-shell {}\" {} teahouse-stranger",
        key.blob_b64(),
        key.openssh()
    ))
}

/// Lines for OpenSSH: every member key, forced into `teahouse-relay shell`.
/// With `only`, prints just that key's line (for `AuthorizedKeysCommand %k`).
pub fn authorized_keys_lines(
    root: &Root,
    binary: &str,
    only: Option<&PublicKey>,
) -> Result<Vec<String>> {
    let root_s = root.dir_str()?;
    let mut keys = BTreeSet::new();
    for id in root.house_ids()? {
        let Ok(h) = root.house(&id) else { continue };
        match h.members() {
            Ok(m) => keys.extend(m.members.into_iter().map(|x| x.key)),
            Err(e) => eprintln!("teahouse-relay: warning: skipping {id}: {e}"),
        }
    }
    Ok(keys
        .into_iter()
        .filter(|k| only.map_or(true, |o| o == k))
        .map(|k| {
            format!(
                "restrict,command=\"{binary} --root {root_s} shell {}\" {} teahouse",
                k.blob_b64(),
                k.openssh()
            )
        })
        .collect())
}

const BEGIN: &str = "# BEGIN TEAHOUSE (managed by teahouse-relay; do not edit)";
const END: &str = "# END TEAHOUSE";

/// Rewrites the teahouse block of an authorized_keys file, keeping every
/// other line, atomically.
pub fn sync_keys(root: &Root, binary: &str, file: &str) -> Result<()> {
    let lines = authorized_keys_lines(root, binary, None)?;
    let path = Path::new(file);
    let existing = fs::read_to_string(path).unwrap_or_default();
    let mut out = String::new();
    let mut inside = false;
    for l in existing.lines() {
        if l == BEGIN {
            inside = true;
            continue;
        }
        if l == END {
            inside = false;
            continue;
        }
        if !inside {
            out.push_str(l);
            out.push('\n');
        }
    }
    out.push_str(BEGIN);
    out.push('\n');
    for l in &lines {
        out.push_str(l);
        out.push('\n');
    }
    out.push_str(END);
    out.push('\n');
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    write_atomic(path, out.as_bytes(), 0o600)?;
    audit::log(
        root,
        Entry::new("keys.synced")
            .field("file", file.to_string())
            .field("keys", lines.len().to_string()),
    );
    Ok(())
}

/// Development transport: lets `git` talk to a local relay through the exact
/// forced-command path OpenSSH would use, without running sshd.
///
/// Use as `GIT_SSH_COMMAND="teahouse-relay --root R local-ssh"` with
/// `GIT_SSH_VARIANT=simple` and URLs like `ssh://alice@local/wutong`; the
/// key is read from `$TEAHOUSE_LOCAL_KEYS/alice.pub`.
///
/// It proves nothing about who you are — never expose it on a real relay.
pub fn local_ssh(root: &Root, args: &[String]) -> Result<()> {
    use std::os::unix::process::CommandExt;
    if args.len() < 2 {
        bail!("local-ssh: expected <user@host> <command>");
    }
    let command = &args[args.len() - 1];
    let host = &args[args.len() - 2];
    let user = host.split_once('@').map(|(u, _)| u).unwrap_or(host);
    if !valid_member_id(user) {
        bail!("local-ssh: bad user {user:?}");
    }
    let dir = std::env::var("TEAHOUSE_LOCAL_KEYS").map_err(|_| {
        crate::error::Error(
            "local-ssh: set TEAHOUSE_LOCAL_KEYS to a directory of <name>.pub files".into(),
        )
    })?;
    let key = read_key_arg(&format!("@{dir}/{user}.pub"))?;
    let exe = std::env::current_exe()?;
    let err = std::process::Command::new(exe)
        .arg("--root")
        .arg(&root.dir)
        .arg("shell")
        .arg(key.blob_b64())
        .env("SSH_ORIGINAL_COMMAND", command)
        .exec();
    bail!("local-ssh: exec failed: {err}")
}
