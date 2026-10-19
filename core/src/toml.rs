//! A strict parser for the small TOML subset used by `members.toml`.
//!
//! Supported: comments, `key = value` pairs, `[table]`, `[[array-of-tables]]`,
//! basic strings (`"..."` with escapes), integers and booleans.
//! Everything else (inline tables, arrays, dotted keys, literal or multi-line
//! strings, floats, dates) is rejected with a line-numbered error rather than
//! silently misread. Rejecting is the safe default for a file that decides who
//! may write to a teahouse.

use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Str(String),
    Int(i64),
    Bool(bool),
}

pub type Table = BTreeMap<String, Value>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Document {
    pub root: Table,
    pub tables: BTreeMap<String, Table>,
    pub arrays: BTreeMap<String, Vec<Table>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub msg: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.msg)
    }
}

impl std::error::Error for ParseError {}

enum Target {
    Root,
    Table(String),
    Array(String),
}

fn err<T>(line: usize, msg: impl Into<String>) -> Result<T, ParseError> {
    Err(ParseError {
        line,
        msg: msg.into(),
    })
}

fn is_bare_key(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

pub fn parse(input: &str) -> Result<Document, ParseError> {
    let mut doc = Document::default();
    let mut target = Target::Root;
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);

    for (idx, raw) in input.lines().enumerate() {
        let line_no = idx + 1;
        let line = strip_comment(raw, line_no)?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("[[") {
            let Some(name) = rest.strip_suffix("]]") else {
                return err(line_no, "unterminated [[array-of-tables]] header");
            };
            let name = name.trim();
            if !is_bare_key(name) {
                return err(line_no, format!("invalid table name {name:?}"));
            }
            if doc.tables.contains_key(name) {
                return err(line_no, format!("{name:?} already defined as a table"));
            }
            doc.arrays
                .entry(name.to_string())
                .or_default()
                .push(Table::new());
            target = Target::Array(name.to_string());
            continue;
        }

        if let Some(rest) = line.strip_prefix('[') {
            let Some(name) = rest.strip_suffix(']') else {
                return err(line_no, "unterminated [table] header");
            };
            let name = name.trim();
            if !is_bare_key(name) {
                return err(line_no, format!("invalid table name {name:?}"));
            }
            if doc.tables.contains_key(name) || doc.arrays.contains_key(name) {
                return err(line_no, format!("table {name:?} defined twice"));
            }
            doc.tables.insert(name.to_string(), Table::new());
            target = Target::Table(name.to_string());
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            return err(line_no, "expected `key = value`");
        };
        let key = key.trim();
        if !is_bare_key(key) {
            return err(
                line_no,
                format!("invalid key {key:?} (only bare keys are allowed)"),
            );
        }
        let value = parse_value(value.trim(), line_no)?;

        let table = match &target {
            Target::Root => &mut doc.root,
            Target::Table(n) => doc.tables.get_mut(n).expect("table exists"),
            Target::Array(n) => doc
                .arrays
                .get_mut(n)
                .and_then(|v| v.last_mut())
                .expect("array entry exists"),
        };
        if table.contains_key(key) {
            return err(line_no, format!("duplicate key {key:?}"));
        }
        table.insert(key.to_string(), value);
    }
    Ok(doc)
}

/// Removes a trailing `# comment`, respecting `#` inside strings.
fn strip_comment(line: &str, line_no: usize) -> Result<&str, ParseError> {
    let mut in_str = false;
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        if in_str {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
        } else if c == '#' {
            return Ok(&line[..i]);
        }
    }
    if in_str {
        return err(line_no, "unterminated string");
    }
    Ok(line)
}

fn parse_value(s: &str, line_no: usize) -> Result<Value, ParseError> {
    if s.is_empty() {
        return err(line_no, "missing value");
    }
    if s.starts_with("\"\"\"") || s.starts_with('\'') {
        return err(line_no, "only basic \"...\" strings are supported");
    }
    if let Some(body) = s.strip_prefix('"') {
        return parse_basic_string(body, line_no).map(Value::Str);
    }
    match s {
        "true" => return Ok(Value::Bool(true)),
        "false" => return Ok(Value::Bool(false)),
        _ => {}
    }
    let digits = s.strip_prefix(['+', '-']).unwrap_or(s);
    if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
        return s
            .parse::<i64>()
            .map(Value::Int)
            .or_else(|_| err(line_no, "integer out of range"));
    }
    err(line_no, format!("unsupported value {s:?}"))
}

fn parse_basic_string(body: &str, line_no: usize) -> Result<String, ParseError> {
    let mut out = String::new();
    let mut chars = body.chars();
    loop {
        let Some(c) = chars.next() else {
            return err(line_no, "unterminated string");
        };
        match c {
            '"' => {
                let rest: String = chars.collect();
                if !rest.trim().is_empty() {
                    return err(line_no, "unexpected characters after string");
                }
                return Ok(out);
            }
            '\\' => {
                let Some(e) = chars.next() else {
                    return err(line_no, "unterminated escape");
                };
                match e {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    'u' => {
                        let hex: String = chars.by_ref().take(4).collect();
                        if hex.len() != 4 {
                            return err(line_no, "short \\u escape");
                        }
                        let code = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32);
                        match code {
                            Some(ch) => out.push(ch),
                            None => return err(line_no, "invalid \\u escape"),
                        }
                    }
                    other => return err(line_no, format!("unknown escape \\{other}")),
                }
            }
            c if (c as u32) < 0x20 && c != '\t' => {
                return err(line_no, "control character in string");
            }
            c => out.push(c),
        }
    }
}

/// Encodes a string as a TOML basic string literal.
pub fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 || c == '\u{7f}' => {
                out.push_str(&format!("\\u{:04X}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_subset() {
        let d = parse(
            "version = 1 # c\n[teahouse]\nname = \"梧桐 # 里\"\n\n[[member]]\nid=\"a\"\nok = true\n[[member]]\nid = \"b\\u0041\\\"\"\n",
        )
        .unwrap();
        assert_eq!(d.root["version"], Value::Int(1));
        assert_eq!(d.tables["teahouse"]["name"], Value::Str("梧桐 # 里".into()));
        assert_eq!(d.arrays["member"].len(), 2);
        assert_eq!(d.arrays["member"][1]["id"], Value::Str("bA\"".into()));
        assert_eq!(d.arrays["member"][0]["ok"], Value::Bool(true));
    }

    #[test]
    fn rejects_what_it_does_not_understand() {
        for bad in [
            "a = [1, 2]",
            "a = {x = 1}",
            "a.b = 1",
            "a = 'lit'",
            "a = \"\"\"x\"\"\"",
            "a = 1.5",
            "a = \"open",
            "a = \"x\" junk",
            "[t]\n[t]",
            "a = 1\na = 2",
            "[[m]]\n[m]",
            "no equals sign",
            "a = \"\\q\"",
        ] {
            assert!(parse(bad).is_err(), "should reject {bad:?}");
        }
    }

    #[test]
    fn quote_roundtrip() {
        for s in [
            "plain",
            "quo\"te",
            "back\\slash",
            "new\nline",
            "茶馆",
            "\u{1}",
        ] {
            let d = parse(&format!("k = {}", quote(s))).unwrap();
            assert_eq!(d.root["k"], Value::Str(s.into()));
        }
    }
}
