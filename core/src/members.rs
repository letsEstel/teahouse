//! `.teahouse/members.toml`: who sits in a teahouse and in which role.
//!
//! Invariants (说明书 第四节):
//! 1. 1 ≤ members ≤ 15
//! 2. exactly one host (老板)
//! 3. at most one steward (伙计)
//! 4. everyone else is a guest (茶客)
//! 5. member ids and keys are unique
//! 6. `kind = "ai"` marks an AI; it is a label, not a rank (any role), and
//!    `agent` appears only on AI members (法第 17 条, docs/MCP.md)
//! 7. a member has 1–5 devices: `key`, plus `[[device]]` tables naming them
//!    (`version = 2`); every key in the chart is unique (法第 18 条,
//!    docs/DEVICES.md)

use crate::key::PublicKey;
use crate::toml::{self, Value};
use std::fmt;

pub const MEMBERS_PATH: &str = ".teahouse/members.toml";
pub const MAX_MEMBERS: usize = 15;
/// The format of a chart without `[[device]]`; it is written whenever
/// nobody has more than one device, so such charts never change.
pub const FORMAT_VERSION: i64 = 1;
/// The format with `[[device]]` tables (docs/DEVICES.md).
pub const FORMAT_VERSION_DEVICES: i64 = 2;
/// Devices (keys) per member, the first included (法第 18 条).
pub const MAX_DEVICES: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Role {
    /// 老板 — full write access, merges proposals, manages seats.
    Host,
    /// 伙计 — writes problems/works/board directly; designated heir.
    Steward,
    /// 茶客 — reads everything, writes only proposals.
    Guest,
}

impl Role {
    pub fn parse(s: &str) -> Option<Role> {
        match s {
            "host" => Some(Role::Host),
            "steward" => Some(Role::Steward),
            "guest" => Some(Role::Guest),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Role::Host => "host",
            Role::Steward => "steward",
            Role::Guest => "guest",
        }
    }

    pub fn zh(self) -> &'static str {
        match self {
            Role::Host => "老板",
            Role::Steward => "伙计",
            Role::Guest => "茶客",
        }
    }
}

/// Whether a seat is held by a person or an AI (docs/MCP.md §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Kind {
    #[default]
    Human,
    Ai,
}

impl Kind {
    pub fn parse(s: &str) -> Option<Kind> {
        match s {
            "human" => Some(Kind::Human),
            "ai" => Some(Kind::Ai),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Human => "human",
            Kind::Ai => "ai",
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub role: Role,
    pub key: PublicKey,
    /// `human` unless the chart says `kind = "ai"`.
    pub kind: Kind,
    /// What kind of AI (`agent = "Claude Code"`), for display only.
    pub agent: Option<String>,
    /// Further devices, beyond the one in `key` (`[[device]]`).
    pub devices: Vec<Device>,
}

/// One more device of a member: its key, and a name people recognise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub name: Option<String>,
    pub key: PublicKey,
}

impl Member {
    /// A person's seat (`kind` human, no `agent`).
    pub fn human(
        id: impl Into<String>,
        name: impl Into<String>,
        role: Role,
        key: PublicKey,
    ) -> Member {
        Member {
            id: id.into(),
            name: name.into(),
            role,
            key,
            kind: Kind::Human,
            agent: None,
            devices: Vec::new(),
        }
    }

    /// Every key of this member: `key` first, then the devices.
    pub fn keys(&self) -> impl Iterator<Item = &PublicKey> {
        std::iter::once(&self.key).chain(self.devices.iter().map(|d| &d.key))
    }

    pub fn has_key(&self, key: &PublicKey) -> bool {
        self.keys().any(|k| k == key)
    }

    /// The name of the device with this key (`None` for an unnamed one).
    pub fn device_name(&self, key: &PublicKey) -> Option<&str> {
        self.devices
            .iter()
            .find(|d| &d.key == key)
            .and_then(|d| d.name.as_deref())
    }

    /// Removes the device with this key; the next device takes the place of
    /// the first one if that is the one removed. Refuses the last device.
    pub fn remove_key(&mut self, key: &PublicKey) -> Result<(), MembersError> {
        if !self.has_key(key) {
            return e(format!(
                "{} is not a device of {:?}",
                key.fingerprint(),
                self.id
            ));
        }
        if self.devices.is_empty() {
            return e(format!(
                "this is the last device of {:?}; a member keeps at least one / 每个成员至少保留一台设备",
                self.id
            ));
        }
        if &self.key == key {
            self.key = self.devices.remove(0).key;
        } else {
            self.devices.retain(|d| &d.key != key);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Members {
    pub name: String,
    pub signboard: Option<String>,
    pub members: Vec<Member>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembersError(pub String);

impl fmt::Display for MembersError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "members.toml: {}", self.0)
    }
}

impl std::error::Error for MembersError {}

fn e<T>(msg: impl Into<String>) -> Result<T, MembersError> {
    Err(MembersError(msg.into()))
}

/// Member ids: lowercase ASCII, digits, `-` and `_`; 1–32 chars; starts alnum.
/// They appear in ref names (`refs/proposals/<id>/…`) so they must be boring.
pub fn valid_member_id(id: &str) -> bool {
    let b = id.as_bytes();
    (1..=32).contains(&b.len())
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter()
            .all(|&c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-' || c == b'_')
}

fn take_str(t: &toml::Table, key: &str, ctx: &str) -> Result<String, MembersError> {
    match t.get(key) {
        Some(Value::Str(s)) => Ok(s.clone()),
        Some(_) => e(format!("{ctx}.{key} must be a string")),
        None => e(format!("{ctx}.{key} is required")),
    }
}

fn check_keys(t: &toml::Table, allowed: &[&str], ctx: &str) -> Result<(), MembersError> {
    for k in t.keys() {
        if !allowed.contains(&k.as_str()) {
            return e(format!("unknown field {ctx}.{k}"));
        }
    }
    Ok(())
}

fn check_text(s: &str, field: &str, max_chars: usize) -> Result<(), MembersError> {
    if s.trim().is_empty() {
        return e(format!("{field} must not be empty"));
    }
    if s.chars().count() > max_chars {
        return e(format!("{field} is longer than {max_chars} characters"));
    }
    if s.chars().any(|c| c.is_control() && c != '\n') {
        return e(format!("{field} contains control characters"));
    }
    Ok(())
}

impl Members {
    pub fn parse(text: &str) -> Result<Members, MembersError> {
        let doc = toml::parse(text).map_err(|err| MembersError(err.to_string()))?;
        check_keys(&doc.root, &["version"], "(root)")?;
        let version = match doc.root.get("version") {
            Some(Value::Int(v @ (FORMAT_VERSION | FORMAT_VERSION_DEVICES))) => *v,
            Some(Value::Int(v)) => return e(format!("unsupported format version {v}")),
            _ => return e("version = 1 is required"),
        };
        for name in doc.tables.keys() {
            if name != "teahouse" {
                return e(format!("unknown table [{name}]"));
            }
        }
        for name in doc.arrays.keys() {
            match name.as_str() {
                "member" => {}
                "device" if version == FORMAT_VERSION_DEVICES => {}
                "device" => return e("[[device]] needs version = 2"),
                _ => return e(format!("unknown table [[{name}]]")),
            }
        }
        let Some(house) = doc.tables.get("teahouse") else {
            return e("[teahouse] table is required");
        };
        check_keys(house, &["name", "signboard"], "teahouse")?;
        let name = take_str(house, "name", "teahouse")?;
        let signboard = match house.get("signboard") {
            None => None,
            Some(Value::Str(s)) => Some(s.clone()),
            Some(_) => return e("teahouse.signboard must be a string"),
        };

        let mut members = Vec::new();
        for (i, t) in doc.arrays.get("member").into_iter().flatten().enumerate() {
            let ctx = format!("member[{i}]");
            check_keys(t, &["id", "name", "role", "key", "kind", "agent"], &ctx)?;
            let id = take_str(t, "id", &ctx)?;
            let mname = take_str(t, "name", &ctx)?;
            let role_s = take_str(t, "role", &ctx)?;
            let role = Role::parse(&role_s).ok_or_else(|| {
                MembersError(format!(
                    "{ctx}.role {role_s:?} must be host, steward or guest"
                ))
            })?;
            let key = PublicKey::parse(&take_str(t, "key", &ctx)?)
                .map_err(|err| MembersError(format!("{ctx}.key: {err}")))?;
            let kind = match t.get("kind") {
                None => Kind::Human,
                Some(Value::Str(k)) => Kind::parse(k)
                    .ok_or_else(|| MembersError(format!("{ctx}.kind {k:?} must be human or ai")))?,
                Some(_) => return e(format!("{ctx}.kind must be a string")),
            };
            let agent = match t.get("agent") {
                None => None,
                Some(Value::Str(a)) => Some(a.clone()),
                Some(_) => return e(format!("{ctx}.agent must be a string")),
            };
            members.push(Member {
                id,
                name: mname,
                role,
                key,
                kind,
                agent,
                devices: Vec::new(),
            });
        }
        for (i, t) in doc.arrays.get("device").into_iter().flatten().enumerate() {
            let ctx = format!("device[{i}]");
            check_keys(t, &["member", "name", "key"], &ctx)?;
            let owner = take_str(t, "member", &ctx)?;
            let name = match t.get("name") {
                None => None,
                Some(Value::Str(n)) => Some(n.clone()),
                Some(_) => return e(format!("{ctx}.name must be a string")),
            };
            let key = PublicKey::parse(&take_str(t, "key", &ctx)?)
                .map_err(|err| MembersError(format!("{ctx}.key: {err}")))?;
            let Some(m) = members.iter_mut().find(|m| m.id == owner) else {
                return e(format!("{ctx}.member {owner:?} has no seat"));
            };
            m.devices.push(Device { name, key });
        }

        let m = Members {
            name,
            signboard,
            members,
        };
        m.validate()?;
        Ok(m)
    }

    /// Checks every invariant. Called on parse and before serialising.
    pub fn validate(&self) -> Result<(), MembersError> {
        check_text(&self.name, "teahouse.name", 64)?;
        if let Some(s) = &self.signboard {
            check_text(s, "teahouse.signboard", 280)?;
        }
        let n = self.members.len();
        if n == 0 {
            return e("a teahouse needs at least one member (the host)");
        }
        if n > MAX_MEMBERS {
            return e(format!(
                "{n} members; a teahouse has at most {MAX_MEMBERS} seats"
            ));
        }
        let hosts = self.members.iter().filter(|m| m.role == Role::Host).count();
        if hosts != 1 {
            return e(format!("exactly one host is required, found {hosts}"));
        }
        let stewards = self
            .members
            .iter()
            .filter(|m| m.role == Role::Steward)
            .count();
        if stewards > 1 {
            return e(format!("at most one steward is allowed, found {stewards}"));
        }
        for (i, m) in self.members.iter().enumerate() {
            if !valid_member_id(&m.id) {
                return e(format!(
                    "member id {:?} is invalid (lowercase letters, digits, '-', '_'; max 32)",
                    m.id
                ));
            }
            check_text(&m.name, &format!("member {:?} name", m.id), 64)?;
            match (&m.agent, m.kind) {
                (Some(_), Kind::Human) => {
                    return e(format!(
                        "member {:?}: `agent` is only for AI members (kind = \"ai\")",
                        m.id
                    ))
                }
                (Some(a), Kind::Ai) => check_text(a, &format!("member {:?} agent", m.id), 64)?,
                (None, _) => {}
            }
            // 法第 18 条: 1–5 devices, named devices told apart.
            if 1 + m.devices.len() > MAX_DEVICES {
                return e(format!(
                    "member {:?} has {} devices; at most {MAX_DEVICES} / 每人最多 {MAX_DEVICES} 台设备",
                    m.id,
                    1 + m.devices.len()
                ));
            }
            for (j, d) in m.devices.iter().enumerate() {
                if let Some(n) = &d.name {
                    check_text(n, &format!("member {:?} device name", m.id), 32)?;
                    if m.devices[..j].iter().any(|o| o.name.as_ref() == Some(n)) {
                        return e(format!("member {:?} has two devices named {n:?}", m.id));
                    }
                }
            }
            for other in &self.members[..i] {
                if other.id == m.id {
                    return e(format!("duplicate member id {:?}", m.id));
                }
            }
        }
        // Every key, of every device, belongs to exactly one seat.
        let all: Vec<(&str, &PublicKey)> = self
            .members
            .iter()
            .flat_map(|m| m.keys().map(move |k| (m.id.as_str(), k)))
            .collect();
        for (i, (id, k)) in all.iter().enumerate() {
            if let Some((other, _)) = all[..i].iter().find(|(_, o)| o == k) {
                if other == id {
                    return e(format!("member {id:?} lists the same key twice"));
                }
                return e(format!("members {other:?} and {id:?} share the same key"));
            }
        }
        Ok(())
    }

    /// The member any of whose devices has this key.
    pub fn by_key(&self, key: &PublicKey) -> Option<&Member> {
        self.members.iter().find(|m| m.has_key(key))
    }

    pub fn by_id(&self, id: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.id == id)
    }

    pub fn host(&self) -> &Member {
        self.members
            .iter()
            .find(|m| m.role == Role::Host)
            .expect("validated: exactly one host")
    }

    pub fn steward(&self) -> Option<&Member> {
        self.members.iter().find(|m| m.role == Role::Steward)
    }

    /// Canonical serialisation. Validates first; never writes a broken file.
    pub fn to_toml(&self) -> Result<String, MembersError> {
        self.validate()?;
        let q = toml::quote;
        let mut s = String::new();
        s.push_str("# 茶馆成员表 · Teahouse members\n");
        s.push_str("# Only the host may change this file. The relay enforces it.\n");
        let has_devices = self.members.iter().any(|m| !m.devices.is_empty());
        let version = if has_devices {
            FORMAT_VERSION_DEVICES
        } else {
            FORMAT_VERSION
        };
        s.push_str(&format!("version = {version}\n\n[teahouse]\n"));
        s.push_str(&format!("name = {}\n", q(&self.name)));
        if let Some(sb) = &self.signboard {
            s.push_str(&format!("signboard = {}\n", q(sb)));
        }
        for m in &self.members {
            s.push_str("\n[[member]]\n");
            s.push_str(&format!("id = {}\n", q(&m.id)));
            s.push_str(&format!("name = {}\n", q(&m.name)));
            s.push_str(&format!("role = {}\n", q(m.role.as_str())));
            if m.kind == Kind::Ai {
                s.push_str("kind = \"ai\"\n");
            }
            if let Some(a) = &m.agent {
                s.push_str(&format!("agent = {}\n", q(a)));
            }
            s.push_str(&format!("key = {}\n", q(&m.key.openssh())));
        }
        for m in &self.members {
            for d in &m.devices {
                s.push_str("\n[[device]]\n");
                s.push_str(&format!("member = {}\n", q(&m.id)));
                if let Some(n) = &d.name {
                    s.push_str(&format!("name = {}\n", q(n)));
                }
                s.push_str(&format!("key = {}\n", q(&d.key.openssh())));
            }
        }
        Ok(s)
    }

    /// 继承: the steward becomes host; the previous host keeps a guest seat.
    pub fn succeed(&self) -> Result<Members, MembersError> {
        let Some(steward) = self.steward() else {
            return e("this teahouse has no steward, so there is no heir");
        };
        let steward_id = steward.id.clone();
        let mut next = self.clone();
        for m in &mut next.members {
            if m.id == steward_id {
                m.role = Role::Host;
            } else if m.role == Role::Host {
                m.role = Role::Guest;
            }
        }
        next.validate()?;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(n: u8) -> PublicKey {
        PublicKey::from_point(&[n; 32])
    }

    fn member(id: &str, role: Role, n: u8) -> Member {
        Member::human(id, id.to_uppercase(), role, key(n))
    }

    fn sample() -> Members {
        Members {
            name: "梧桐里三号院".into(),
            signboard: Some("车位、垃圾、楼道灯，咱们自己管。".into()),
            members: vec![
                member("chenli", Role::Host, 1),
                member("zhou", Role::Steward, 2),
                member("wang", Role::Guest, 3),
            ],
        }
    }

    #[test]
    fn roundtrip() {
        let m = sample();
        let text = m.to_toml().unwrap();
        assert_eq!(Members::parse(&text).unwrap(), m);
    }

    #[test]
    fn invariants() {
        let mut m = sample();
        m.members.clear();
        assert!(m.validate().is_err(), "zero members");

        let mut m = sample();
        for i in 0..13 {
            m.members
                .push(member(&format!("g{i}"), Role::Guest, 10 + i));
        }
        assert_eq!(m.members.len(), 16);
        assert!(m.validate().is_err(), "16 members");
        m.members.pop();
        assert!(m.validate().is_ok(), "15 members is fine");

        let mut m = sample();
        m.members[2].role = Role::Host;
        assert!(m.validate().is_err(), "two hosts");

        let mut m = sample();
        m.members[0].role = Role::Guest;
        assert!(m.validate().is_err(), "no host");

        let mut m = sample();
        m.members[2].role = Role::Steward;
        assert!(m.validate().is_err(), "two stewards");

        let mut m = sample();
        m.members[2].id = "zhou".into();
        assert!(m.validate().is_err(), "duplicate id");

        let mut m = sample();
        m.members[2].key = key(1);
        assert!(m.validate().is_err(), "shared key");

        for bad in [
            "",
            "Upper",
            "-x",
            "has space",
            "a/b",
            "x".repeat(33).as_str(),
        ] {
            let mut m = sample();
            m.members[2].id = bad.into();
            assert!(m.validate().is_err(), "bad id {bad:?}");
        }
    }

    #[test]
    fn strict_schema() {
        let good = sample().to_toml().unwrap();
        assert!(Members::parse(&good.replace("version = 1", "version = 3")).is_err());
        assert!(Members::parse(&good.replace("version = 1\n", "")).is_err());
        assert!(Members::parse(&format!("{good}\nadmin = true\n")).is_err());
        assert!(Members::parse(&good.replace("role = \"guest\"", "role = \"admin\"")).is_err());
        assert!(Members::parse(&good.replace("[teahouse]", "[teahouse]\nowner = \"x\"")).is_err());
        assert!(Members::parse(&format!("{good}\n[extra]\n")).is_err());
    }

    #[test]
    fn succession() {
        let next = sample().succeed().unwrap();
        assert_eq!(next.host().id, "zhou");
        assert_eq!(next.by_id("chenli").unwrap().role, Role::Guest);
        assert!(next.steward().is_none());

        let mut no_heir = sample();
        no_heir.members.retain(|m| m.role != Role::Steward);
        assert!(no_heir.succeed().is_err());
    }

    #[test]
    fn ai_members_are_marked_in_any_role() {
        let k = |n: u8| key(n).openssh();
        let chart = |host_extra: &str, extra: &str| {
            format!(
                "version = 1\n[teahouse]\nname = \"开发组\"\n\n[[member]]\nid = \"u\"\nname = \"用户\"\nrole = \"host\"\n{host_extra}key = \"{}\"\n\n[[member]]\nid = \"ai-2\"\nname = \"2 号\"\n{extra}key = \"{}\"\n",
                k(1),
                k(2)
            )
        };
        let m = Members::parse(&chart(
            "",
            "role = \"guest\"\nkind = \"ai\"\nagent = \"Claude Code\"\n",
        ))
        .unwrap();
        let ai = m.by_id("ai-2").unwrap();
        assert_eq!(
            (ai.kind, ai.agent.as_deref()),
            (Kind::Ai, Some("Claude Code"))
        );
        assert_eq!(m.host().kind, Kind::Human);
        // Round trip, and humans carry no kind line.
        let t = m.to_toml().unwrap();
        assert_eq!(Members::parse(&t).unwrap(), m);
        assert_eq!(t.matches("kind = ").count(), 1);

        // 法第 17 条: a label, not a rank. An AI may be the steward, or the host.
        assert!(Members::parse(&chart("", "role = \"steward\"\nkind = \"ai\"\n")).is_ok());
        let ai_host = Members::parse(&chart("kind = \"ai\"\n", "role = \"guest\"\n")).unwrap();
        assert_eq!(ai_host.host().kind, Kind::Ai);
        // …and an AI steward inherits like anyone.
        let m2 = Members::parse(&chart("", "role = \"steward\"\nkind = \"ai\"\n")).unwrap();
        assert_eq!(m2.succeed().unwrap().host().id, "ai-2");

        for (bad, why) in [
            (
                "role = \"guest\"\nagent = \"Claude Code\"\n",
                "only for AI members",
            ),
            ("role = \"guest\"\nkind = \"robot\"\n", "human or ai"),
            ("role = \"guest\"\nkind = 1\n", "must be a string"),
        ] {
            let err = Members::parse(&chart("", bad)).unwrap_err();
            assert!(err.0.contains(why), "{bad}: {err}");
        }
    }

    #[test]
    fn devices() {
        let k = |n: u8| key(n).openssh();
        let base = format!(
            "version = 2\n[teahouse]\nname = \"梧桐里\"\n\n[[member]]\nid = \"wang\"\nname = \"王阿姨\"\nrole = \"host\"\nkey = \"{}\"\n\n[[member]]\nid = \"li\"\nname = \"李\"\nrole = \"guest\"\nkey = \"{}\"\n",
            k(1),
            k(2)
        );
        let dev = |member: &str, name: Option<&str>, n: u8| {
            let name = name
                .map(|x| format!("name = \"{x}\"\n"))
                .unwrap_or_default();
            format!(
                "\n[[device]]\nmember = \"{member}\"\n{name}key = \"{}\"\n",
                k(n)
            )
        };
        let m =
            Members::parse(&(base.clone() + &dev("wang", Some("手机"), 3) + &dev("wang", None, 4)))
                .unwrap();
        let wang = m.by_id("wang").unwrap();
        assert_eq!(wang.keys().count(), 3);
        assert_eq!(
            m.by_key(&key(3)).unwrap().id,
            "wang",
            "any device names the member"
        );
        assert_eq!(wang.device_name(&key(3)), Some("手机"));
        // Round trip; a chart with devices is version 2, without them version 1.
        let t = m.to_toml().unwrap();
        assert!(t.contains("version = 2") && t.contains("[[device]]"));
        assert_eq!(Members::parse(&t).unwrap(), m);
        let mut plain = m.clone();
        plain.members[0].devices.clear();
        assert!(
            plain.to_toml().unwrap().starts_with("# 茶馆成员表")
                && plain.to_toml().unwrap().contains("version = 1")
        );

        // Removing: another device can go; the first one is replaced by the
        // next; the last one stays.
        let mut w = wang.clone();
        w.remove_key(&key(1)).unwrap();
        assert_eq!((w.key.clone(), w.devices.len()), (key(3), 1));
        w.remove_key(&key(4)).unwrap();
        let err = w.remove_key(&key(3)).unwrap_err();
        assert!(err.0.contains("last device"), "{err}");
        assert!(w.remove_key(&key(9)).is_err());

        for (bad, why) in [
            (
                base.replace("version = 2", "version = 1") + &dev("wang", None, 3),
                "needs version = 2",
            ),
            (base.clone() + &dev("zhao", None, 3), "has no seat"),
            (base.clone() + &dev("wang", None, 2), "share the same key"),
            (base.clone() + &dev("wang", None, 1), "same key twice"),
            (
                base.clone() + &dev("wang", Some("手机"), 3) + &dev("wang", Some("手机"), 4),
                "two devices named",
            ),
            (
                base.clone()
                    + &dev("wang", None, 3)
                    + &dev("wang", None, 4)
                    + &dev("wang", None, 5)
                    + &dev("wang", None, 6)
                    + &dev("wang", None, 7),
                "at most 5",
            ),
            (
                base.clone() + "\n[[device]]\nmember = \"wang\"\ncolor = \"red\"\nkey = \"x\"\n",
                "unknown field",
            ),
        ] {
            let err = Members::parse(&bad).unwrap_err();
            assert!(err.0.contains(why), "{why}: {err}");
        }
        // Five devices is fine.
        let five = base
            + &dev("wang", None, 3)
            + &dev("wang", None, 4)
            + &dev("wang", None, 5)
            + &dev("wang", None, 6);
        assert_eq!(
            Members::parse(&five)
                .unwrap()
                .by_id("wang")
                .unwrap()
                .keys()
                .count(),
            5
        );
    }
}
