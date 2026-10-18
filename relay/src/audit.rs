//! Append-only audit log: one JSON object per line.
//!
//! Each line is written with a single `write` on an `O_APPEND` descriptor, so
//! concurrent pushes cannot interleave inside a line. Failing to log never
//! blocks a decision that was already made, but it is reported on stderr.

use crate::root::Root;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Entry<'a> {
    pub event: &'a str,
    pub house: Option<&'a str>,
    pub member: Option<&'a str>,
    pub key: Option<&'a str>,
    pub fields: Vec<(&'a str, String)>,
}

impl<'a> Entry<'a> {
    pub fn new(event: &'a str) -> Self {
        Entry {
            event,
            house: None,
            member: None,
            key: None,
            fields: Vec::new(),
        }
    }
    pub fn house(mut self, h: &'a str) -> Self {
        self.house = Some(h);
        self
    }
    pub fn member(mut self, m: &'a str) -> Self {
        self.member = Some(m);
        self
    }
    pub fn key(mut self, k: &'a str) -> Self {
        self.key = Some(k);
        self
    }
    pub fn field(mut self, k: &'a str, v: impl Into<String>) -> Self {
        self.fields.push((k, v.into()));
        self
    }
}

pub fn json_str(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

pub fn render(e: &Entry<'_>, now: SystemTime) -> String {
    let mut parts = vec![
        format!("\"ts\":{}", json_str(&rfc3339(now))),
        format!("\"event\":{}", json_str(e.event)),
        format!("\"pid\":{}", std::process::id()),
    ];
    if let Some(h) = e.house {
        parts.push(format!("\"house\":{}", json_str(h)));
    }
    if let Some(m) = e.member {
        parts.push(format!("\"member\":{}", json_str(m)));
    }
    if let Some(k) = e.key {
        parts.push(format!("\"key\":{}", json_str(k)));
    }
    for (k, v) in &e.fields {
        parts.push(format!("{}:{}", json_str(k), json_str(v)));
    }
    format!("{{{}}}\n", parts.join(","))
}

pub fn log(root: &Root, e: Entry<'_>) {
    let line = render(&e, SystemTime::now());
    let path = root.log_path();
    let res = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| f.write_all(line.as_bytes()));
    if let Err(err) = res {
        eprintln!(
            "teahouse-relay: warning: cannot write audit log {}: {err}",
            path.display()
        );
    }
}

/// UTC timestamp, e.g. `2026-10-18T21:30:05Z`.
pub fn rfc3339(t: SystemTime) -> String {
    let secs = t
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem / 60) % 60,
        rem % 60
    )
}

/// Howard Hinnant's days→civil algorithm (proleptic Gregorian).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn timestamps() {
        assert_eq!(rfc3339(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        assert_eq!(
            rfc3339(UNIX_EPOCH + Duration::from_secs(951_782_400)),
            "2000-02-29T00:00:00Z"
        );
        assert_eq!(
            rfc3339(UNIX_EPOCH + Duration::from_secs(1_792_358_405)),
            "2026-10-18T21:20:05Z"
        );
    }

    #[test]
    fn json_escaping() {
        assert_eq!(json_str("a\"b\\c\n\u{1}茶"), "\"a\\\"b\\\\c\\n\\u0001茶\"");
        let line = render(
            &Entry::new("push")
                .house("w")
                .field("ref", "refs/heads/main"),
            UNIX_EPOCH,
        );
        assert!(line.starts_with("{\"ts\":\"1970-01-01T00:00:00Z\",\"event\":\"push\""));
        assert!(line.ends_with("\"ref\":\"refs/heads/main\"}\n"));
    }
}
