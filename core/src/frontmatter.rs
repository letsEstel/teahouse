//! Frontmatter (CLIENT-API.md §1.1): one level of `key: value` between
//! `---` lines at the top of a file, and the attachment references of §1.6
//! that use it. Shared by the relay (which checks references) and the
//! client engine (which reads and writes them).

/// `key: value` pairs between `---` lines at the very top of a file, and the
/// rest of the file.
///
/// Only one level of `key: value` is understood (no nesting, no lists). A
/// value may be double-quoted with JSON escapes or single-quoted with `''`
/// for a quote; unquoted, ` #` starts a comment. Keys that repeat keep their
/// last value. A file without a closing `---` has no frontmatter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frontmatter {
    pub fields: Vec<(String, String)>,
}

impl Frontmatter {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .filter(|v| !v.is_empty())
    }
}

pub fn split_frontmatter(text: &str) -> (Frontmatter, &str) {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let Some(rest) = strip_fence(text) else {
        return (Frontmatter::default(), text);
    };
    let mut fm = Frontmatter::default();
    let mut pos = 0;
    while pos < rest.len() {
        let end = rest[pos..].find('\n').map_or(rest.len(), |n| pos + n + 1);
        let line = &rest[pos..end];
        let bare = line.trim_end_matches(['\n', '\r']);
        if bare.trim_end() == "---" {
            return (fm, &rest[end..]);
        }
        if let Some((k, v)) = parse_field(bare) {
            fm.fields.push((k, v));
        }
        pos = end;
    }
    (Frontmatter::default(), text)
}

fn strip_fence(text: &str) -> Option<&str> {
    let first = text.find('\n')?;
    (text[..first].trim_end() == "---").then(|| &text[first + 1..])
}

fn parse_field(line: &str) -> Option<(String, String)> {
    if line.starts_with([' ', '\t', '#', '-']) {
        return None;
    }
    let (k, v) = line.split_once(':')?;
    let k = k.trim();
    if k.is_empty()
        || !k
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }
    Some((k.to_string(), parse_value(v)))
}

fn parse_value(v: &str) -> String {
    let v = v.trim();
    if let Some(q) = v.strip_prefix('"') {
        let mut out = String::new();
        let mut cs = q.chars();
        while let Some(c) = cs.next() {
            match c {
                '"' => return out,
                '\\' => match cs.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some('u') => {
                        let hex: String = cs.by_ref().take(4).collect();
                        if let Some(c) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32)
                        {
                            out.push(c);
                        }
                    }
                    Some(o) => out.push(o),
                    None => {}
                },
                o => out.push(o),
            }
        }
        return out;
    }
    if let Some(q) = v.strip_prefix('\'') {
        let mut out = String::new();
        let mut cs = q.chars().peekable();
        while let Some(c) = cs.next() {
            if c == '\'' {
                if cs.peek() == Some(&'\'') {
                    cs.next();
                    out.push('\'');
                } else {
                    return out;
                }
            } else {
                out.push(c);
            }
        }
        return out;
    }
    if v.starts_with('#') {
        return String::new();
    }
    let cut = v
        .match_indices('#')
        .find(|(i, _)| v[..*i].ends_with([' ', '\t']))
        .map_or(v.len(), |(i, _)| i);
    v[..cut].trim().to_string()
}

/// A frontmatter value as the engine writes it: bare when that reads back
/// the same, double-quoted otherwise.
pub fn quote_value(v: &str) -> String {
    let bare_ok = !v.is_empty()
        && v.trim() == v
        && !v.contains('#')
        && !v.contains(':')
        && !v.starts_with([
            '"', '\'', '-', '[', '{', '&', '*', '!', '|', '>', '%', '@', '`',
        ])
        && !v.chars().any(char::is_control);
    if bare_ok {
        v.to_string()
    } else {
        json_string(v)
    }
}

/// A frontmatter block for `fields` (values quoted as needed), ending in a
/// newline.
pub fn render_frontmatter(fields: &[(&str, &str)]) -> String {
    let mut s = String::from("---\n");
    for (k, v) in fields {
        s.push_str(&format!("{k}: {}\n", quote_value(v)));
    }
    s.push_str("---\n");
    s
}

/// `text` with frontmatter fields set (`Some`) or removed (`None`), keeping
/// every other line, comment and the body as they were.
pub fn set_fields(text: &str, changes: &[(&str, Option<&str>)]) -> String {
    let (fm, body) = split_frontmatter(text);
    if fm.fields.is_empty() && body.len() == text.len() && !text.starts_with("---") {
        let fields: Vec<(&str, &str)> = changes
            .iter()
            .filter_map(|(k, v)| v.map(|v| (*k, v)))
            .collect();
        return render_frontmatter(&fields) + text;
    }
    let head = &text[..text.len() - body.len()];
    let mut lines: Vec<String> = head.lines().map(str::to_string).collect();
    // lines[0] is the opening fence, the last one the closing fence.
    let close = lines.len().saturating_sub(1);
    let mut done = vec![false; changes.len()];
    let mut out = vec![lines[0].clone()];
    for line in &lines[1..close] {
        let key = parse_field(line.trim_end_matches('\r')).map(|(k, _)| k);
        match key.and_then(|k| changes.iter().position(|(c, _)| *c == k)) {
            Some(i) => {
                if let (Some(v), false) = (changes[i].1, done[i]) {
                    out.push(format!("{}: {}", changes[i].0, quote_value(v)));
                }
                done[i] = true;
            }
            None => out.push(line.clone()),
        }
    }
    for (i, (k, v)) in changes.iter().enumerate() {
        if let (Some(v), false) = (v, done[i]) {
            out.push(format!("{k}: {}", quote_value(v)));
        }
    }
    out.push(lines.remove(close));
    out.join("\n") + "\n" + body
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobRef {
    pub title: Option<String>,
    /// The hex digest, without the `sha256:` prefix.
    pub hash: String,
    pub size: u64,
    pub mime: String,
    pub purged: bool,
    pub note: String,
}

pub fn parse_blob_ref(text: &str) -> Option<BlobRef> {
    let (fm, body) = split_frontmatter(text);
    let hash = fm
        .get("blob")?
        .strip_prefix("sha256:")?
        .to_ascii_lowercase();
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some(BlobRef {
        title: fm.get("title").map(str::to_string),
        hash,
        size: fm.get("size").and_then(|s| s.parse().ok()).unwrap_or(0),
        mime: fm
            .get("type")
            .unwrap_or("application/octet-stream")
            .to_string(),
        purged: fm.get("purged").is_some(),
        note: body.trim().to_string(),
    })
}

/// Largest attachment (§1.6): 100 MiB.
pub const MAX_BLOB_BYTES: u64 = 100 * 1024 * 1024;

/// The relay's check of an attachment reference: `blob: sha256:<64 hex>`,
/// `size` (bytes, at most 100 MiB) and `type` are required.
pub fn check_blob_ref(text: &str) -> Result<BlobRef, String> {
    let (fm, _) = split_frontmatter(text);
    let blob = fm
        .get("blob")
        .ok_or("has no `blob: sha256:…` line / 附件引用缺少 blob")?;
    let hash = blob.strip_prefix("sha256:").unwrap_or("");
    if !crate::sha256::valid_hex(hash) {
        return Err(format!(
            "`blob: {blob}` must be sha256: and 64 lowercase hex digits / blob 格式不对"
        ));
    }
    let size: u64 = fm
        .get("size")
        .and_then(|s| s.parse().ok())
        .ok_or("needs `size:` in bytes / 附件引用缺少 size")?;
    if size > MAX_BLOB_BYTES {
        return Err(format!(
            "attachment of {size} bytes; the limit is 100 MiB / 附件最大 100 MiB"
        ));
    }
    let mime = fm.get("type").ok_or("needs `type:` / 附件引用缺少 type")?;
    if !mime.contains('/') {
        return Err(format!("`type: {mime}` is not a media type"));
    }
    Ok(parse_blob_ref(text).expect("checked above"))
}

/// A JSON string literal for `s` (quotes included).
fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
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
    fn frontmatter() {
        let (fm, body) = split_frontmatter(
            "---\ntitle: 小区停车位不好找\nstatus: active            # open | active\nowner: wang\n---\n## 现状\n院内\n",
        );
        assert_eq!(fm.get("title"), Some("小区停车位不好找"));
        assert_eq!(fm.get("status"), Some("active"));
        assert_eq!(fm.get("owner"), Some("wang"));
        assert_eq!(body, "## 现状\n院内\n");

        let (fm, _) = split_frontmatter(
            "---\r\ntitle: \"修 #3 号楼: \\\"东\\\"\\u4e00\"\r\nb: 'it''s'\r\nc: a#b\r\nd: # nothing\r\n---\r\n",
        );
        assert_eq!(fm.get("title"), Some("修 #3 号楼: \"东\"一"));
        assert_eq!(fm.get("b"), Some("it's"));
        assert_eq!(fm.get("c"), Some("a#b"));
        assert_eq!(fm.get("d"), None);

        // No closing fence: no frontmatter, the whole text is the body.
        let t = "---\ntitle: x\n";
        assert_eq!(split_frontmatter(t), (Frontmatter::default(), t));
        let t = "plain\n---\n";
        assert_eq!(split_frontmatter(t), (Frontmatter::default(), t));
    }

    #[test]
    fn setting_fields() {
        let t = "---\ntitle: 停车\nstatus: open   # 注释\nowner: wang\n---\n## 现状\n";
        assert_eq!(
            set_fields(
                t,
                &[
                    ("status", Some("active")),
                    ("owner", None),
                    ("x", Some("a: b"))
                ]
            ),
            "---\ntitle: 停车\nstatus: active\nx: \"a: b\"\n---\n## 现状\n"
        );
        assert_eq!(
            set_fields("正文\n", &[("status", Some("open"))]),
            "---\nstatus: open\n---\n正文\n"
        );
        assert_eq!(
            render_frontmatter(&[("title", "修 #3 号楼"), ("kind", "plan")]),
            "---\ntitle: \"修 #3 号楼\"\nkind: plan\n---\n"
        );
    }

    #[test]
    fn quoting_round_trips() {
        for v in [
            "小区停车位不好找",
            "修 #3 号楼",
            "a: b",
            " padded ",
            "\"quoted\"",
            "- list?",
            "it's",
            "",
            "line\nbreak",
        ] {
            let text = format!("---\nk: {}\n---\n", quote_value(v));
            let (fm, _) = split_frontmatter(&text);
            assert_eq!(fm.fields, vec![("k".to_string(), v.to_string())], "{text}");
        }
        assert_eq!(quote_value("小区停车位"), "小区停车位");
    }

    #[test]
    fn blob_refs() {
        let ok = format!(
            "---\nblob: \"sha256:{}\"\nsize: 9\ntype: image/png\n---\n",
            "ab".repeat(32)
        );
        assert_eq!(check_blob_ref(&ok).unwrap().size, 9);
        for bad in [
            "---\nsize: 9\ntype: image/png\n---\n".to_string(),
            ok.replace("size: 9", "size: lots"),
            ok.replace("size: 9", "size: 104857601"),
            ok.replace("type: image/png\n", ""),
            ok.replace("ab", "AB"),
            ok.replace("sha256:", "md5:"),
        ] {
            assert!(check_blob_ref(&bad).is_err(), "{bad}");
        }
        let r = parse_blob_ref(&format!(
            "---\ntitle: 车位照片\nblob: sha256:{}\nsize: 482113\ntype: image/png\n---\n说明\n",
            "9F".repeat(32)
        ))
        .unwrap();
        assert_eq!(r.hash, "9f".repeat(32));
        assert_eq!(
            (r.size, r.mime.as_str(), r.purged),
            (482113, "image/png", false)
        );
        assert_eq!(r.note, "说明");
        assert!(parse_blob_ref("---\nblob: sha256:abc\n---\n").is_none());
    }
}
