//! Several devices per member (docs/DEVICES.md, 法则第 18–19 条).
//!
//! * `pair [--name N] [--minutes M]`: a seated key asks for a pairing code
//!   (`TH-…`, one-time, 10 minutes by default, at most 60). Only its hash is
//!   kept, in `<root>/pairs/`.
//! * Redeeming it (`redeem <code> --name <device>`, also from redeem-shell)
//!   adds the new key as a device of the member the issuing key sits as, in
//!   **every** teahouse of this relay where that key has a seat.
//! * `unpair <house> <device>`: a member removes another of their own
//!   devices, by name or `SHA256:` fingerprint. Not the device in use, and
//!   never the last one.
//!
//! The chart changes are written by the relay (`invites::edit_chart`):
//! authored by the member, committed and signed by Teahouse Relay, checked
//! by the same laws as a push.

use crate::audit::{self, Entry};
use crate::bail;
use crate::error::{Error, Result};
use crate::invites::{
    check_name, code_hash, display, edit_chart, now, random_code, ChartEdit, Lock, Seated,
};
use crate::root::{members_at, Root};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use teahouse_core::members::{Device, Member, MAX_DEVICES};
use teahouse_core::rules::MAIN_REF;
use teahouse_core::PublicKey;

pub const DEFAULT_MINUTES: u64 = 10;
pub const MAX_MINUTES: u64 = 60;
/// Pending pairing codes per key.
const MAX_PENDING: usize = 3;

fn pairs_dir(root: &Root) -> PathBuf {
    root.dir.join("pairs")
}

/// A pending pairing code as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Pair {
    prefix: String,
    /// The issuing key (base64 blob).
    by: String,
    name: Option<String>,
    expires: i64,
}

impl Pair {
    fn render(&self) -> String {
        let mut s = format!(
            "prefix = {}\nby = {}\nexpires = {}\n",
            self.prefix, self.by, self.expires
        );
        if let Some(n) = &self.name {
            s.push_str(&format!("name = {}\n", n.replace('\n', " ")));
        }
        s
    }

    fn parse(text: &str) -> Option<Pair> {
        let m: BTreeMap<&str, &str> = text.lines().filter_map(|l| l.split_once(" = ")).collect();
        Some(Pair {
            prefix: m.get("prefix")?.to_string(),
            by: m.get("by")?.to_string(),
            name: m.get("name").map(|s| s.to_string()),
            expires: m.get("expires")?.parse().ok()?,
        })
    }
}

/// The teahouses where `key` sits, with its seat in each.
fn seats_of(root: &Root, key: &PublicKey) -> Result<Vec<(String, Member)>> {
    let mut out = Vec::new();
    for id in root.house_ids()? {
        let Ok(h) = root.house(&id) else { continue };
        let Ok(m) = h.members() else { continue };
        if let Some(me) = m.by_key(key) {
            out.push((id, me.clone()));
        }
    }
    Ok(out)
}

/// A pairing code for `key`'s seats (`pair`).
pub fn create(
    root: &Root,
    key: &PublicKey,
    name: Option<&str>,
    minutes: Option<u64>,
) -> Result<String> {
    if seats_of(root, key)?.is_empty() {
        bail!("this key has no seat on this relay / 这把钥匙在这个 Relay 上没有座位");
    }
    if let Some(n) = name {
        check_name(n)?;
    }
    let minutes = minutes.unwrap_or(DEFAULT_MINUTES);
    if !(1..=MAX_MINUTES).contains(&minutes) {
        bail!("a pairing code lasts 1 to {MAX_MINUTES} minutes / 配对码有效期是 1 到 {MAX_MINUTES} 分钟");
    }
    let _lock = Lock::take(root)?;
    let dir = pairs_dir(root);
    fs::create_dir_all(&dir)?;
    // Expired codes go; a key keeps at most a few pending.
    let mut mine = 0;
    for e in fs::read_dir(&dir)?.filter_map(|e| e.ok()) {
        let Some(p) = fs::read_to_string(e.path())
            .ok()
            .and_then(|t| Pair::parse(&t))
        else {
            continue;
        };
        if now() > p.expires {
            let _ = fs::remove_file(e.path());
        } else if p.by == key.blob_b64() {
            mine += 1;
        }
    }
    if mine >= MAX_PENDING {
        bail!("{MAX_PENDING} pairing codes are already pending for this device; use one or wait / 待用的配对码太多");
    }
    let code = random_code()?;
    let pair = Pair {
        prefix: code[..4].to_string(),
        by: key.blob_b64().to_string(),
        name: name.map(|n| n.trim().to_string()),
        expires: now() + minutes as i64 * 60,
    };
    crate::root::write_atomic(
        &dir.join(format!("{}.pair", code_hash(&code))),
        pair.render().as_bytes(),
        0o600,
    )?;
    audit::log(
        root,
        Entry::new("pair.created")
            .key(&key.short())
            .field("pair", pair.prefix.clone())
            .field("minutes", minutes.to_string()),
    );
    Ok(display(&code))
}

/// A device name not yet used by this member: `name`, `name 2`, …
fn free_name(member: &Member, name: &str) -> String {
    let taken = |n: &str| member.devices.iter().any(|d| d.name.as_deref() == Some(n));
    if !taken(name) {
        return name.to_string();
    }
    (2..)
        .map(|i| format!("{name} {i}"))
        .find(|n| !taken(n))
        .expect("a free name")
}

/// Redeems a pairing code for the new device `key`, under the invitation
/// lock (`invites::redeem` calls this). `Ok(None)`: not a pairing code.
pub fn redeem_pair(
    root: &Root,
    key: &PublicKey,
    code: &str,
    device: &str,
) -> Result<Option<Seated>> {
    let file = pairs_dir(root).join(format!("{}.pair", code_hash(code)));
    let Some(pair) = fs::read_to_string(&file).ok().and_then(|t| Pair::parse(&t)) else {
        return Ok(None);
    };
    if now() > pair.expires {
        fs::remove_file(&file)?;
        bail!("this pairing code has expired; make a new one on your other device / 配对码已过期，请在另一台设备上重新生成");
    }
    let issuer = PublicKey::from_blob_b64(&pair.by)?;
    let name = if device.trim().is_empty() {
        pair.name.clone().unwrap_or_else(|| "设备".into())
    } else {
        device.trim().to_string()
    };
    check_name(&name)?;
    let seats = seats_of(root, &issuer)?;
    if seats.is_empty() {
        fs::remove_file(&file)?;
        bail!(
            "the device that made this code has no seat any more / 生成配对码的设备已经没有座位了"
        );
    }
    // Check every teahouse before changing any, so a refusal changes nothing.
    for (house, _) in &seats {
        let h = root.house(house)?;
        let m = members_at(&h.git, MAIN_REF)?;
        if let Some(other) = m.by_key(key) {
            bail!(
                "this device already sits in {house} as {:?} / 这台设备已经在 {house} 里了",
                other.id
            );
        }
        let me = m
            .by_key(&issuer)
            .ok_or_else(|| Error("seat changed meanwhile; try again".into()))?;
        if me.keys().count() >= MAX_DEVICES {
            bail!("{:?} already has {MAX_DEVICES} devices in {house}; remove one first / 每人最多 {MAX_DEVICES} 台设备", me.id);
        }
    }
    let mut done = Vec::new();
    for (house, _) in &seats {
        let member = edit_chart(root, house, "teahouse: device paired", |members| {
            let me = members
                .by_key(&issuer)
                .ok_or_else(|| Error("seat changed meanwhile; try again".into()))?;
            if members.by_key(key).is_some() {
                bail!("this device already sits in {house}");
            }
            let dev_name = free_name(me, &name);
            let mut next = members.clone();
            let m = next
                .members
                .iter_mut()
                .find(|m| m.id == me.id)
                .expect("present");
            m.devices.push(Device {
                name: Some(dev_name.clone()),
                key: key.clone(),
            });
            Ok(ChartEdit {
                author: me.id.clone(),
                message: format!(
                    "{} 添加设备：{dev_name}\n\nProblem: meta\nAct: seat\nDevice: add {dev_name}\nPair: {}\n",
                    me.name, pair.prefix
                ),
                next,
                value: me.id.clone(),
            })
        })?;
        done.push((house.clone(), member));
    }
    fs::remove_file(&file)?;
    for (house, member) in &done {
        audit::log(
            root,
            Entry::new("pair.redeemed")
                .house(house)
                .member(member)
                .key(&key.short())
                .field("pair", pair.prefix.clone()),
        );
    }
    let mut it = done.into_iter();
    let (house, member) = it.next().expect("at least one seat");
    Ok(Some(Seated {
        house,
        member,
        more: it.collect(),
    }))
}

/// `unpair <house> <device>`: the member behind `key` removes another of
/// their own devices, named or by `SHA256:` fingerprint.
pub fn unpair(root: &Root, key: &PublicKey, house: &str, which: &str) -> Result<String> {
    let _lock = Lock::take(root)?;
    let removed = edit_chart(root, house, "teahouse: device removed", |members| {
        let me = members.by_key(key).ok_or_else(|| {
            Error(
                "teahouse not found, or you have no seat in it / 茶馆不存在，或者你在这里没有座位"
                    .into(),
            )
        })?;
        let target = me
            .keys()
            .find(|k| k.fingerprint() == which || me.device_name(k) == Some(which))
            .cloned()
            .ok_or_else(|| {
                Error(format!(
                    "{which:?} is not one of your devices (give its name or SHA256: fingerprint) / 不是你的设备"
                ))
            })?;
        if &target == key {
            bail!("this is the device you are using; remove it from another of your devices / 不能移除正在用的这台设备");
        }
        let label = me
            .device_name(&target)
            .map(str::to_string)
            .unwrap_or_else(|| target.fingerprint());
        let mut next = members.clone();
        let m = next
            .members
            .iter_mut()
            .find(|m| m.id == me.id)
            .expect("present");
        m.remove_key(&target).map_err(|e| Error(e.to_string()))?;
        Ok(ChartEdit {
            author: me.id.clone(),
            message: format!(
                "{} 移除设备：{label}\n\nProblem: meta\nAct: seat\nDevice: remove {label}\n",
                me.name
            ),
            next,
            value: (me.id.clone(), label, target.short()),
        })
    })?;
    audit::log(
        root,
        Entry::new("device.removed")
            .house(house)
            .member(&removed.0)
            .key(&removed.2)
            .field("device", removed.1.clone()),
    );
    Ok(removed.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_form() {
        let p = Pair {
            prefix: "7KQ2".into(),
            by: "AAAA".into(),
            name: Some("手机 = 新".into()),
            expires: 9,
        };
        assert_eq!(Pair::parse(&p.render()), Some(p));
    }
}
