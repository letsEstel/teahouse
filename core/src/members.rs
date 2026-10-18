//! `.teahouse/members.toml`: who sits in a teahouse and in which role.
//!
//! Invariants (说明书 第四节):
//! 1. 1 ≤ members ≤ 15
//! 2. exactly one host (老板)
//! 3. at most one steward (伙计)
//! 4. everyone else is a guest (茶客)
//! 5. member ids and keys are unique

use crate::key::PublicKey;
use crate::toml::{self, Value};
use std::fmt;

pub const MEMBERS_PATH: &str = ".teahouse/members.toml";
pub const MAX_MEMBERS: usize = 15;
pub const FORMAT_VERSION: i64 = 1;

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
        match doc.root.get("version") {
            Some(Value::Int(FORMAT_VERSION)) => {}
            Some(Value::Int(v)) => return e(format!("unsupported format version {v}")),
            _ => return e("version = 1 is required"),
        }
        for name in doc.tables.keys() {
            if name != "teahouse" {
                return e(format!("unknown table [{name}]"));
            }
        }
        for name in doc.arrays.keys() {
            if name != "member" {
                return e(format!("unknown table [[{name}]]"));
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
            check_keys(t, &["id", "name", "role", "key"], &ctx)?;
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
            members.push(Member {
                id,
                name: mname,
                role,
                key,
            });
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
            for other in &self.members[..i] {
                if other.id == m.id {
                    return e(format!("duplicate member id {:?}", m.id));
                }
                if other.key == m.key {
                    return e(format!(
                        "members {:?} and {:?} share the same key",
                        other.id, m.id
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn by_key(&self, key: &PublicKey) -> Option<&Member> {
        self.members.iter().find(|m| &m.key == key)
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
        s.push_str(&format!("version = {FORMAT_VERSION}\n\n[teahouse]\n"));
        s.push_str(&format!("name = {}\n", q(&self.name)));
        if let Some(sb) = &self.signboard {
            s.push_str(&format!("signboard = {}\n", q(sb)));
        }
        for m in &self.members {
            s.push_str("\n[[member]]\n");
            s.push_str(&format!("id = {}\n", q(&m.id)));
            s.push_str(&format!("name = {}\n", q(&m.name)));
            s.push_str(&format!("role = {}\n", q(m.role.as_str())));
            s.push_str(&format!("key = {}\n", q(&m.key.openssh())));
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
        Member {
            id: id.into(),
            name: id.to_uppercase(),
            role,
            key: key(n),
        }
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
        assert!(Members::parse(&good.replace("version = 1", "version = 2")).is_err());
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
}
