//! Invitations (docs/INVITE.md): the host creates a one-time code; whoever
//! redeems it gets a seat, written by the relay on the host's behalf.
//!
//! * A **founding** invitation (`teahouse-relay found`, role host) names a
//!   teahouse that does not exist yet; redeeming it opens that teahouse
//!   with the redeemer as its host, so nobody has to handle a public key.
//! * Codes are 80 random bits in Crockford Base32, shown as
//!   `TH-XXXX-XXXX-XXXX-XXXX`. Only their SHA-256 is stored, in
//!   `<root>/invites/<house>/<hash>.inv`; the code itself is printed once.
//! * Redeeming is all or nothing: the seat commit is checked by the same
//!   laws as a push (`hook::check_update`, with the host as the pusher) and
//!   lands with a compare-and-swap on `main`; the invitation is then spent.
//! * The seat commit is authored by the host, committed by Teahouse Relay
//!   and signed with the relay's own key (`<root>/relay_signing_key`), with
//!   `Problem: meta`, `Act: seat` and `Invite: <first four>` trailers.

use crate::audit::{self, Entry};
use crate::bail;
use crate::error::{Error, Result};
use crate::git::{commit_files_as, Signature};
use crate::hook::{self, Context, Update};
use crate::root::{members_at, Root};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;
use teahouse_core::members::{valid_member_id, Member, Members, Role, MAX_MEMBERS, MEMBERS_PATH};
use teahouse_core::rules::MAIN_REF;
use teahouse_core::PublicKey;

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
/// Code length in Base32 characters (80 bits).
pub const CODE_LEN: usize = 16;
pub const DEFAULT_DAYS: u64 = 7;
pub const MAX_DAYS: u64 = 30;
/// Pending invitations per teahouse.
const MAX_PENDING: usize = 50;
/// Failed redemptions per key per hour, and attempts per minute overall.
const MAX_FAILS_PER_KEY_HOUR: usize = 5;
const MAX_ATTEMPTS_PER_MINUTE: usize = 30;

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// `TH-XXXX-XXXX-XXXX-XXXX` for 16 code characters.
pub fn display(code: &str) -> String {
    let groups: Vec<&str> = (0..code.len())
        .step_by(4)
        .map(|i| &code[i..(i + 4).min(code.len())])
        .collect();
    format!("TH-{}", groups.join("-"))
}

/// The 16 code characters of what someone typed or pasted: case, dashes and
/// spaces do not matter, `O`/`I`/`L` read as `0`/`1`/`1` (Crockford), and a
/// `TH-` prefix or `@relay` suffix is ignored.
pub fn normalize(input: &str) -> Option<String> {
    let s = input.trim();
    let s = s.split('@').next().unwrap_or(s);
    let s = s.to_ascii_uppercase();
    let s = s.strip_prefix("TH").unwrap_or(&s);
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '-' | ' ' => {}
            'O' => out.push('0'),
            'I' | 'L' => out.push('1'),
            c if ALPHABET.contains(&(c as u8)) && c.is_ascii() => out.push(c),
            _ => return None,
        }
    }
    (out.len() == CODE_LEN).then_some(out)
}

fn random_code() -> Result<String> {
    let mut bytes = [0u8; 10];
    fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    let mut bits: u128 = 0;
    for b in bytes {
        bits = bits << 8 | b as u128;
    }
    Ok((0..CODE_LEN)
        .rev()
        .map(|i| ALPHABET[((bits >> (i * 5)) & 31) as usize] as char)
        .collect())
}

fn code_hash(code: &str) -> String {
    teahouse_core::sha256::hex(format!("teahouse-invite:{code}").as_bytes())
}

/// A pending invitation as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invite {
    pub prefix: String,
    pub role: Role,
    pub name: Option<String>,
    pub created: i64,
    pub expires: i64,
    pub by: String,
    /// Founding invitations: the teahouse's name and signboard.
    pub house_name: Option<String>,
    pub signboard: Option<String>,
}

impl Invite {
    fn render(&self) -> String {
        let mut s = format!(
            "prefix = {}\nrole = {}\ncreated = {}\nexpires = {}\nby = {}\n",
            self.prefix, self.role, self.created, self.expires, self.by
        );
        for (k, v) in [
            ("name", &self.name),
            ("house_name", &self.house_name),
            ("signboard", &self.signboard),
        ] {
            if let Some(v) = v {
                s.push_str(&format!("{k} = {}\n", v.replace('\n', " ")));
            }
        }
        s
    }

    fn parse(text: &str) -> Option<Invite> {
        let m: BTreeMap<&str, &str> = text.lines().filter_map(|l| l.split_once(" = ")).collect();
        Some(Invite {
            prefix: m.get("prefix")?.to_string(),
            role: Role::parse(m.get("role")?)?,
            name: m.get("name").map(|s| s.to_string()),
            created: m.get("created")?.parse().ok()?,
            expires: m.get("expires")?.parse().ok()?,
            by: m.get("by")?.to_string(),
            house_name: m.get("house_name").map(|s| s.to_string()),
            signboard: m.get("signboard").map(|s| s.to_string()),
        })
    }
}

/// A short-lived lock over invitation state and relay-written commits, so
/// two redemptions never race for the last seat. `mkdir` is atomic.
struct Lock(PathBuf);

impl Lock {
    fn take(root: &Root) -> Result<Lock> {
        let dir = root.dir.join("invites");
        fs::create_dir_all(&dir)?;
        let path = dir.join(".lock");
        for _ in 0..200 {
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Lock(path)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    // A crashed holder leaves the lock behind; it is stale
                    // after a minute.
                    let stale = fs::metadata(&path)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.elapsed().ok())
                        .is_some_and(|age| age > Duration::from_secs(60));
                    if stale {
                        let _ = fs::remove_dir(&path);
                    } else {
                        std::thread::sleep(Duration::from_millis(50));
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
        bail!("the relay is busy; try again / Relay 正忙，请稍后再试")
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

/// Creates an invitation and returns the code (`TH-…`), shown only once.
pub fn create(
    root: &Root,
    house: &str,
    host: &Member,
    role: Role,
    name: Option<&str>,
    days: Option<u64>,
) -> Result<String> {
    if role == Role::Host {
        bail!("an invitation can seat a guest or the steward, not a host / 邀请只能是茶客或伙计");
    }
    let days = days.unwrap_or(DEFAULT_DAYS);
    if !(1..=MAX_DAYS).contains(&days) {
        bail!("an invitation lasts 1 to {MAX_DAYS} days / 有效期是 1 到 {MAX_DAYS} 天");
    }
    if let Some(n) = name {
        check_name(n)?;
    }
    let _lock = Lock::take(root)?;
    let dir = root.invites_dir(house);
    fs::create_dir_all(&dir)?;
    let pending = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".inv"))
        .count();
    if pending >= MAX_PENDING {
        bail!("{MAX_PENDING} invitations are already pending; revoke some first / 待用的邀请太多");
    }
    let code = random_code()?;
    let t = now();
    let inv = Invite {
        prefix: code[..4].to_string(),
        role,
        name: name.map(str::to_string),
        created: t,
        expires: t + days as i64 * 86_400,
        by: host.id.clone(),
        house_name: None,
        signboard: None,
    };
    crate::root::write_atomic(
        &dir.join(format!("{}.inv", code_hash(&code))),
        inv.render().as_bytes(),
        0o600,
    )?;
    audit::log(
        root,
        Entry::new("invite.created")
            .house(house)
            .member(&host.id)
            .field("invite", inv.prefix.clone())
            .field("role", role.as_str())
            .field("days", days.to_string()),
    );
    Ok(display(&code))
}

/// An operator's founding invitation: whoever redeems it opens teahouse
/// `house` as its host. Returns the code, shown only once.
pub fn create_founding(
    root: &Root,
    house: &str,
    name: &str,
    signboard: Option<&str>,
    days: Option<u64>,
) -> Result<String> {
    if !crate::root::valid_house_id(house) {
        bail!("teahouse id {house:?} must be lowercase letters, digits and '-' (max 63)");
    }
    if root.house(house).is_ok() {
        bail!("teahouse {house:?} already exists / 茶馆 {house} 已经开张了");
    }
    let days = days.unwrap_or(DEFAULT_DAYS);
    if !(1..=MAX_DAYS).contains(&days) {
        bail!("an invitation lasts 1 to {MAX_DAYS} days / 有效期是 1 到 {MAX_DAYS} 天");
    }
    // The chart it will open with must be valid; check the name and
    // signboard now rather than at redemption.
    Members {
        name: name.to_string(),
        signboard: signboard.map(str::to_string),
        members: Vec::new(),
    }
    .validate()
    .or_else(|e| {
        if e.0.contains("at least one member") {
            Ok(())
        } else {
            Err(Error(e.to_string()))
        }
    })?;
    root.load_config()?;
    let _lock = Lock::take(root)?;
    let dir = root.invites_dir(house);
    fs::create_dir_all(&dir)?;
    let code = random_code()?;
    let t = now();
    let inv = Invite {
        prefix: code[..4].to_string(),
        role: Role::Host,
        name: None,
        created: t,
        expires: t + days as i64 * 86_400,
        by: "operator".into(),
        house_name: Some(name.to_string()),
        signboard: signboard.map(str::to_string),
    };
    crate::root::write_atomic(
        &dir.join(format!("{}.inv", code_hash(&code))),
        inv.render().as_bytes(),
        0o600,
    )?;
    audit::log(
        root,
        Entry::new("invite.created")
            .house(house)
            .field("invite", inv.prefix.clone())
            .field("role", "host")
            .field("days", days.to_string()),
    );
    Ok(display(&code))
}

/// Revokes the pending invitation whose code starts with `prefix`.
pub fn revoke(root: &Root, house: &str, host: &Member, prefix: &str) -> Result<()> {
    let prefix = normalize(&format!("{prefix}000000000000"))
        .map(|c| c[..4].to_string())
        .ok_or_else(|| {
            Error("give the first four characters of the code / 请给出邀请码的前四位".into())
        })?;
    let _lock = Lock::take(root)?;
    let dir = root.invites_dir(house);
    let mut found = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.filter_map(|e| e.ok()) {
            let p = e.path();
            if p.extension().is_some_and(|x| x == "inv") {
                if let Some(inv) = fs::read_to_string(&p).ok().and_then(|t| Invite::parse(&t)) {
                    if inv.prefix == prefix {
                        found.push(p);
                    }
                }
            }
        }
    }
    match found.as_slice() {
        [] => bail!("no pending invitation starts with {prefix} / 没有这个邀请"),
        [one] => fs::remove_file(one)?,
        _ => bail!(
            "{} invitations start with {prefix}; give more of the code / 有歧义",
            found.len()
        ),
    }
    audit::log(
        root,
        Entry::new("invite.revoked")
            .house(house)
            .member(&host.id)
            .field("invite", prefix),
    );
    Ok(())
}

fn check_name(name: &str) -> Result<()> {
    let n = name.trim();
    if n.is_empty() || n.chars().count() > 32 || n.chars().any(char::is_control) {
        bail!("a name is 1 to 32 characters / 名字是 1 到 32 个字");
    }
    Ok(())
}

/// A member id for a new seat: the name's ASCII letters and digits if it
/// has any, else `m`; numbered when taken (`wang`, `wang2`; `m1`, `m2`).
pub fn new_member_id(name: &str, taken: &[&str]) -> String {
    let mut base = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            base.push(c.to_ascii_lowercase());
        } else if (c == ' ' || c == '-' || c == '_') && !base.is_empty() && !base.ends_with('-') {
            base.push('-');
        }
    }
    let base: String = base.trim_matches('-').chars().take(24).collect();
    let base = base.trim_end_matches('-').to_string();
    let candidates: Box<dyn Iterator<Item = String>> = if base.is_empty() {
        Box::new((1..).map(|n| format!("m{n}")))
    } else {
        Box::new(std::iter::once(base.clone()).chain((2..).map(move |n| format!("{base}{n}"))))
    };
    let mut candidates = candidates;
    candidates
        .find(|id| valid_member_id(id) && !taken.contains(&id.as_str()))
        .expect("some id is free")
}

/// Attempts kept for rate limiting: `<time> <key> ok|fail` lines, an hour.
fn attempts_file(root: &Root) -> PathBuf {
    root.dir.join("invites").join(".attempts")
}

fn rate_limited(root: &Root, key: &str) -> Result<Option<String>> {
    let t = now();
    let text = fs::read_to_string(attempts_file(root)).unwrap_or_default();
    let mut recent_all = 0;
    let mut fails_key = 0;
    for l in text.lines() {
        let f: Vec<&str> = l.split(' ').collect();
        let [when, k, result] = f.as_slice() else {
            continue;
        };
        let Ok(when) = when.parse::<i64>() else {
            continue;
        };
        if t - when < 60 {
            recent_all += 1;
        }
        if t - when < 3600 && *k == key && *result == "fail" {
            fails_key += 1;
        }
    }
    if fails_key >= MAX_FAILS_PER_KEY_HOUR {
        return Ok(Some("too many failed attempts with this key; try again in an hour / 失败次数太多，请一小时后再试".into()));
    }
    if recent_all >= MAX_ATTEMPTS_PER_MINUTE {
        return Ok(Some("too many invitations are being redeemed right now; try again in a minute / 兑换太频繁，请一分钟后再试".into()));
    }
    Ok(None)
}

fn record_attempt(root: &Root, key: &str, ok: bool) -> Result<()> {
    let t = now();
    let text = fs::read_to_string(attempts_file(root)).unwrap_or_default();
    let mut kept: Vec<&str> = text
        .lines()
        .filter(|l| {
            l.split(' ')
                .next()
                .and_then(|w| w.parse::<i64>().ok())
                .is_some_and(|w| t - w < 3600)
        })
        .collect();
    let line = format!("{t} {key} {}", if ok { "ok" } else { "fail" });
    kept.push(&line);
    crate::root::write_atomic(
        &attempts_file(root),
        (kept.join("\n") + "\n").as_bytes(),
        0o600,
    )
}

/// What a redemption gave.
pub struct Seated {
    pub house: String,
    pub member: String,
}

/// Redeems `code` for `key` with the name `name`. Every failure is
/// recorded and counted against the key.
pub fn redeem(root: &Root, key: &PublicKey, code: &str, name: &str) -> Result<Seated> {
    let _lock = Lock::take(root)?;
    let short = key.short();
    if let Some(why) = rate_limited(root, &short)? {
        audit::log(
            root,
            Entry::new("invite.failed")
                .key(&short)
                .field("reason", "rate limited"),
        );
        bail!("{why}");
    }
    let result = redeem_locked(root, key, code, name);
    record_attempt(root, &short, result.is_ok())?;
    match &result {
        Ok(s) => audit::log(
            root,
            Entry::new("invite.redeemed")
                .house(&s.house)
                .member(&s.member)
                .key(&short)
                .field(
                    "invite",
                    normalize(code)
                        .map(|c| c[..4].to_string())
                        .unwrap_or_default(),
                ),
        ),
        Err(e) => audit::log(
            root,
            Entry::new("invite.failed")
                .key(&short)
                .field("reason", e.0.clone()),
        ),
    }
    result
}

const NOT_VALID: &str = "this invitation is not valid (mistyped, revoked or already used) / 邀请码无效（输错了、已撤销或已用过）";

/// Finds the stored invitation for a code: `(house, file)`.
fn find(root: &Root, code: &str) -> Result<Option<(String, PathBuf)>> {
    let hash = code_hash(code);
    // Not only existing teahouses: a founding invitation names one to come.
    let Ok(rd) = fs::read_dir(root.dir.join("invites")) else {
        return Ok(None);
    };
    for e in rd.filter_map(|e| e.ok()) {
        let house = e.file_name().to_string_lossy().into_owned();
        if !crate::root::valid_house_id(&house) {
            continue;
        }
        let p = e.path().join(format!("{hash}.inv"));
        if p.is_file() {
            return Ok(Some((house, p)));
        }
    }
    Ok(None)
}

/// Opens the teahouse a founding invitation names, with `key` as its host.
fn found(
    root: &Root,
    house_id: &str,
    file: &std::path::Path,
    inv: &Invite,
    key: &PublicKey,
    name: &str,
) -> Result<Seated> {
    if root.house(house_id).is_ok() {
        fs::remove_file(file)?;
        bail!("teahouse {house_id} is already open; this invitation is spent / 茶馆已经开张，这个邀请已失效");
    }
    let id = new_member_id(name, &[]);
    let house_name = inv
        .house_name
        .clone()
        .unwrap_or_else(|| house_id.to_string());
    crate::admin::create_house(
        root,
        &crate::admin::NewHouse {
            id: house_id,
            name: &house_name,
            signboard: inv.signboard.as_deref(),
            host_id: &id,
            host_name: name.trim(),
            host_key: "",
        },
        key,
        Some(&inv.prefix),
    )?;
    fs::remove_file(file)?;
    if let Ok(cfg) = root.load_config() {
        if let Some(path) = cfg.authorized_keys {
            if let Err(e) = crate::admin::sync_keys(root, &cfg.binary, &path) {
                eprintln!("teahouse-relay: warning: could not refresh {path}: {e}");
            }
        }
    }
    Ok(Seated {
        house: house_id.to_string(),
        member: id,
    })
}

fn redeem_locked(root: &Root, key: &PublicKey, code: &str, name: &str) -> Result<Seated> {
    let code = normalize(code).ok_or_else(|| Error(NOT_VALID.into()))?;
    check_name(name)?;
    let Some((house_id, file)) = find(root, &code)? else {
        bail!("{NOT_VALID}");
    };
    let inv = fs::read_to_string(&file)
        .ok()
        .and_then(|t| Invite::parse(&t))
        .ok_or_else(|| Error(NOT_VALID.into()))?;
    if now() > inv.expires {
        fs::remove_file(&file)?;
        bail!("this invitation has expired; ask the host for a new one / 邀请已过期，请向老板要一个新的");
    }
    if inv.role == Role::Host {
        return found(root, &house_id, &file, &inv, key, name);
    }
    let house = root.house(&house_id)?;
    for _attempt in 0..5 {
        let Some(main) = house.git.resolve(MAIN_REF)? else {
            bail!("teahouse {house_id} has no main");
        };
        let members = members_at(&house.git, &main)?;
        if members.by_key(key).is_some() {
            bail!("this device already has a seat in this teahouse / 这台设备已经在这间茶馆里了");
        }
        if members.members.len() >= MAX_MEMBERS {
            bail!("the teahouse is full ({MAX_MEMBERS} seats) / 茶馆满座了");
        }
        if inv.role == Role::Steward && members.steward().is_some() {
            bail!("the teahouse already has a steward / 茶馆已经有伙计了");
        }
        let taken: Vec<&str> = members.members.iter().map(|m| m.id.as_str()).collect();
        let id = new_member_id(name, &taken);
        let mut next: Members = members.clone();
        next.members.push(Member {
            id: id.clone(),
            name: name.trim().to_string(),
            role: inv.role,
            key: key.clone(),
        });
        let toml = next.to_toml().map_err(|e| Error(e.to_string()))?;
        let host = members.host();
        let message = format!(
            "{} 入座（邀请）\n\nProblem: meta\nAct: seat\nInvite: {}\n",
            name.trim(),
            inv.prefix
        );
        let new = commit_files_as(
            &house.git,
            &root.tmp_dir(),
            Some(&main),
            &[(MEMBERS_PATH, toml.as_bytes())],
            &message,
            &Signature {
                name: &host.name,
                email: &format!("{}@teahouse", host.id),
            },
            &Signature {
                name: "Teahouse Relay",
                email: "relay@teahouse",
            },
            Some(&root.signing_key()),
        )?;
        // The same laws as any push, with the host as the one pushing.
        let blobs = root.blobs_dir(&house_id);
        let cx = Context {
            git: &house.git,
            pusher: host,
            now: now(),
            blobs: &blobs,
            signers: None,
        };
        let problems = hook::check_update(
            &cx,
            &Update {
                old: main.clone(),
                new: new.clone(),
                refname: MAIN_REF.to_string(),
            },
        )?;
        if !problems.is_empty() {
            bail!("the seat change breaks a law: {}", problems.join("; "));
        }
        // Compare-and-swap: a push that landed meanwhile makes this fail,
        // and we start again from the new main.
        let swapped = house
            .git
            .run(&[
                "update-ref",
                "-m",
                "teahouse: invitation redeemed",
                MAIN_REF,
                &new,
                &main,
            ])
            .is_ok();
        if !swapped {
            continue;
        }
        fs::remove_file(&file)?;
        audit::log(
            root,
            Entry::new("seats.changed")
                .house(&house_id)
                .field("invite", inv.prefix.clone()),
        );
        if let Ok(cfg) = root.load_config() {
            if let Some(path) = cfg.authorized_keys {
                if let Err(e) = crate::admin::sync_keys(root, &cfg.binary, &path) {
                    eprintln!("teahouse-relay: warning: could not refresh {path}: {e}");
                }
            }
        }
        return Ok(Seated {
            house: house_id,
            member: id,
        });
    }
    bail!("the teahouse kept changing; try again / 茶馆正在变动，请再试一次")
}

/// The relay's signing key, made by `init` when missing.
pub fn ensure_signing_key(root: &Root) -> Result<()> {
    let key = root.signing_key();
    if key.exists() {
        return Ok(());
    }
    let out = std::process::Command::new("ssh-keygen")
        .args([
            "-q",
            "-t",
            "ed25519",
            "-N",
            "",
            "-C",
            "teahouse-relay",
            "-f",
        ])
        .arg(&key)
        .output();
    match out {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) => bail!(
            "ssh-keygen failed: {}",
            String::from_utf8_lossy(&o.stderr).trim()
        ),
        Err(e) => bail!("cannot run ssh-keygen (needed for the relay's signing key): {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes() {
        let c = random_code().unwrap();
        assert_eq!(c.len(), CODE_LEN);
        let shown = display(&c);
        assert_eq!(shown.len(), 3 + 16 + 3);
        assert_eq!(normalize(&shown).as_deref(), Some(c.as_str()));
        assert_eq!(
            normalize(&format!("{}@relay.example.org:2222", shown.to_lowercase())).as_deref(),
            Some(c.as_str())
        );
        assert_eq!(
            normalize("th-7kq2 m9xf-3rdp-wcsl").as_deref(),
            Some("7KQ2M9XF3RDPWCS1")
        );
        assert_eq!(
            normalize("TH-OOOO-IIII-LLLL-0000").as_deref(),
            Some("0000111111110000")
        );
        assert_eq!(normalize("TH-7KQ2-M9XF-3RDP"), None);
        assert_eq!(
            normalize("TH-7KQ2-M9XF-3RDP-WC5U"),
            None,
            "U is not Crockford"
        );
        assert_ne!(code_hash("A"), code_hash("B"));
    }

    #[test]
    fn member_ids() {
        assert_eq!(new_member_id("王阿姨", &[]), "m1");
        assert_eq!(new_member_id("王阿姨", &["m1", "m2"]), "m3");
        assert_eq!(new_member_id("Wang Li", &[]), "wang-li");
        assert_eq!(new_member_id("wang", &["wang"]), "wang2");
        assert_eq!(new_member_id("小王 Wang", &["wang", "wang2"]), "wang3");
        assert_eq!(new_member_id("---", &[]), "m1");
        assert!(valid_member_id(&new_member_id(&"x".repeat(80), &[])));
    }

    #[test]
    fn stored_form() {
        let inv = Invite {
            prefix: "7KQ2".into(),
            role: Role::Steward,
            name: Some("王阿姨".into()),
            created: 1,
            expires: 2,
            by: "chenli".into(),
            house_name: Some("梧桐里 = 三号院".into()),
            signboard: None,
        };
        assert_eq!(Invite::parse(&inv.render()), Some(inv));
    }
}
