//! The content formats of CLIENT-API.md §1: frontmatter, problems, works,
//! attachment references, notices, posts and commit trailers. Pure functions,
//! no I/O.

use crate::model::{Act, Status, WorkKind};
use teahouse_core::rules::{self, TalkTopic};
use teahouse_core::ProblemRef;

// ---------------------------------------------------------------------------
// Frontmatter

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
        serde_json::to_string(v).expect("strings serialise")
    }
}

// ---------------------------------------------------------------------------
// Problems

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemDoc {
    pub title: String,
    pub status: Status,
    pub owner: Option<String>,
    pub body: Vec<(String, String)>,
}

/// `problems/<id>-<slug>.md` (or `problems/<id>.md`) → `(id, slug)`.
pub fn parse_problem_path(path: &str) -> Option<(&str, &str)> {
    let name = path.strip_prefix("problems/")?;
    if name.contains('/') || name.len() < 7 {
        return None;
    }
    let id = name.get(..4)?;
    if !id.bytes().all(|b| b.is_ascii_digit()) || !ProblemRef::matches_problem_file(id, name) {
        return None;
    }
    let slug = name[4..].trim_end_matches(".md").trim_start_matches('-');
    Some((id, slug))
}

pub fn parse_problem(text: &str, id: &str, slug: &str) -> ProblemDoc {
    let (fm, body) = split_frontmatter(text);
    let title = fm.get("title").map(str::to_string).unwrap_or_else(|| {
        if slug.is_empty() {
            id.to_string()
        } else {
            slug.to_string()
        }
    });
    let status = fm
        .get("status")
        .and_then(Status::parse)
        .unwrap_or(Status::Open);
    ProblemDoc {
        title,
        status,
        owner: fm.get("owner").map(str::to_string),
        body: sections(body),
    }
}

/// Splits a body at its `## ` headings. Text before the first heading becomes
/// a section with an empty heading, if there is any.
pub fn sections(body: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut heading = String::new();
    let mut text = String::new();
    let mut started = false;
    let flush = |out: &mut Vec<(String, String)>, h: &str, t: &str, started: bool| {
        let t = t.trim_matches('\n').trim_end();
        if started || !t.trim().is_empty() {
            out.push((h.to_string(), t.to_string()));
        }
    };
    for line in body.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            flush(&mut out, &heading, &text, started);
            heading = h.trim().to_string();
            text.clear();
            started = true;
        } else {
            text.push_str(line);
            text.push('\n');
        }
    }
    flush(&mut out, &heading, &text, started);
    out
}

/// A short summary of a problem: the first non-empty line of its body,
/// at most 120 characters.
pub fn summary(body: &[(String, String)]) -> String {
    let line = body
        .iter()
        .flat_map(|(_, t)| t.lines())
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    let mut s: String = line.chars().take(120).collect();
    if line.chars().count() > 120 {
        s.push('…');
    }
    s
}

// ---------------------------------------------------------------------------
// Works

/// How a file under `works/<problem>/` is shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkFile {
    /// A work of its own.
    Work(WorkKind),
    /// `<table>.csv.meta.md`: only the title of a table.
    TableMeta,
}

/// Classifies a file under `works/` by name (and, for markdown and
/// references, by its frontmatter).
pub fn classify_work(path: &str, fm: Option<&Frontmatter>) -> WorkFile {
    let lower = path.to_lowercase();
    if lower.ends_with(".ref.md") {
        let image = fm
            .and_then(|f| f.get("type"))
            .is_some_and(|t| t.starts_with("image/"));
        return WorkFile::Work(if image {
            WorkKind::Image
        } else {
            WorkKind::File
        });
    }
    if lower.ends_with(".csv.meta.md") {
        return WorkFile::TableMeta;
    }
    if lower.ends_with(".md") {
        let plan = fm.and_then(|f| f.get("kind")) == Some("plan");
        return WorkFile::Work(if plan { WorkKind::Plan } else { WorkKind::Doc });
    }
    if lower.ends_with(".csv") {
        return WorkFile::Work(WorkKind::Table);
    }
    WorkFile::Work(WorkKind::File)
}

/// `works/<problem>/<rest>` → `(problem, rest)`.
pub fn parse_work_path(path: &str) -> Option<(&str, &str)> {
    let rest = path.strip_prefix("works/")?;
    let (pid, file) = rest.split_once('/')?;
    (pid.len() == 4 && pid.bytes().all(|b| b.is_ascii_digit()) && !file.is_empty())
        .then_some((pid, file))
}

/// The title a file shows when nothing better is written down: its name
/// without directory and known extensions.
pub fn file_title(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    for ext in [".ref.md", ".csv.meta.md", ".md", ".csv"] {
        if name.len() > ext.len() && name.to_lowercase().ends_with(ext) {
            return name[..name.len() - ext.len()].to_string();
        }
    }
    name.to_string()
}

/// An attachment reference (`*.ref.md`, CLIENT-API.md §1.6).
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

// ---------------------------------------------------------------------------
// Board and talk

/// `board/<notice>.md` → notice name.
pub fn parse_notice_path(path: &str) -> Option<&str> {
    let name = path.strip_prefix("board/")?.strip_suffix(".md")?;
    (!name.is_empty() && !name.contains('/')).then_some(name)
}

/// What a post answers, its author and when it says it was written.
pub struct PostPath<'a> {
    pub topic: TalkTopic<'a>,
    pub member: &'a str,
    pub written: i64,
}

pub fn parse_post_path(path: &str) -> Option<PostPath<'_>> {
    let p = rules::parse_talk_path(path)?;
    Some(PostPath {
        topic: p.topic,
        member: p.author,
        written: rules::stamp_epoch(p.stamp)?,
    })
}

// ---------------------------------------------------------------------------
// Commits

/// The act of a commit: its `Act:` trailer, or inferred from what it changed
/// (CLIENT-API.md §1.4) when there is none.
pub fn infer_act(trailer: Option<&str>, changes: &[(char, String)]) -> Act {
    if let Some(a) = trailer.and_then(Act::parse) {
        return a;
    }
    let any = |pred: &dyn Fn(char, &str) -> bool| changes.iter().any(|(c, p)| pred(*c, p));
    if any(&|c, p| c == 'A' && parse_problem_path(p).is_some()) {
        Act::OpenProblem
    } else if any(&|_, p| p == teahouse_core::MEMBERS_PATH) {
        Act::Seat
    } else if any(&|c, p| c == 'A' && p.starts_with("works/")) {
        Act::AddWork
    } else if any(&|_, p| p.starts_with("works/")) {
        Act::UpdateWork
    } else if any(&|c, p| c == 'A' && parse_notice_path(p).is_some()) {
        Act::Notice
    } else if any(&|c, p| c == 'A' && p.starts_with("talk/")) {
        Act::Note
    } else {
        Act::Update
    }
}

/// A commit body without its trailer block (the last paragraph, when every
/// line of it is `Key: value`).
pub fn strip_trailers(body: &str) -> String {
    let body = body.trim_end();
    let (head, last) = match body.rfind("\n\n") {
        Some(i) => (&body[..i], &body[i + 2..]),
        None => ("", body),
    };
    let is_trailer = |l: &str| {
        l.split_once(": ").is_some_and(|(k, _)| {
            !k.is_empty() && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
    };
    if !last.is_empty() && last.lines().all(is_trailer) {
        head.trim_end().to_string()
    } else {
        body.to_string()
    }
}

/// The member a commit's author names. The engine writes
/// `<name> <<member-id>@teahouse>`; commits made by hand are matched by
/// the email's local part, then by name. `None` when nobody matches.
pub fn author_member<'m>(
    members: &'m teahouse_core::Members,
    name: &str,
    email: &str,
) -> Option<&'m str> {
    let local = email.split('@').next().unwrap_or("");
    members
        .by_id(local)
        .or_else(|| members.by_id(name))
        .or_else(|| members.members.iter().find(|m| m.name == name))
        .map(|m| m.id.as_str())
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
    fn problems() {
        assert_eq!(
            parse_problem_path("problems/0001-停车位.md"),
            Some(("0001", "停车位"))
        );
        assert_eq!(parse_problem_path("problems/0002.md"), Some(("0002", "")));
        assert_eq!(parse_problem_path("problems/12-x.md"), None);
        assert_eq!(parse_problem_path("problems/0001-x.txt"), None);
        assert_eq!(parse_problem_path("problems/a/0001-x.md"), None);

        let p = parse_problem(
            "---\ntitle: 停车\nstatus: weird\n---\n开场白\n\n## 现状\n院内 42 个车位\n\n## 目标\n每户一个\n",
            "0001",
            "x",
        );
        assert_eq!(p.title, "停车");
        assert_eq!(p.status, Status::Open);
        assert_eq!(
            p.body,
            vec![
                ("".into(), "开场白".into()),
                ("现状".into(), "院内 42 个车位".into()),
                ("目标".into(), "每户一个".into()),
            ]
        );
        assert_eq!(summary(&p.body), "开场白");
        assert_eq!(parse_problem("", "0003", "").title, "0003");
        assert_eq!(parse_problem("", "0003", "slug").title, "slug");
    }

    #[test]
    fn works() {
        let fm = |s: &str| split_frontmatter(s).0;
        let img = fm("---\ntype: image/png\n---\n");
        let plan = fm("---\nkind: plan\n---\n");
        use WorkKind::*;
        assert_eq!(
            classify_work("works/0001/照片.png.ref.md", Some(&img)),
            WorkFile::Work(Image)
        );
        assert_eq!(
            classify_work("works/0001/合同.pdf.ref.md", None),
            WorkFile::Work(File)
        );
        assert_eq!(
            classify_work("works/0001/t.csv.meta.md", None),
            WorkFile::TableMeta
        );
        assert_eq!(
            classify_work("works/0001/方案.md", Some(&plan)),
            WorkFile::Work(Plan)
        );
        assert_eq!(
            classify_work("works/0001/记录.md", None),
            WorkFile::Work(Doc)
        );
        assert_eq!(
            classify_work("works/0001/t.CSV", None),
            WorkFile::Work(Table)
        );
        assert_eq!(
            classify_work("works/0001/old.jpg", None),
            WorkFile::Work(File)
        );
        assert_eq!(
            parse_work_path("works/0001/a/b.md"),
            Some(("0001", "a/b.md"))
        );
        assert_eq!(parse_work_path("works/x/b.md"), None);
        assert_eq!(file_title("works/0001/照片.png.ref.md"), "照片.png");
        assert_eq!(file_title("works/0001/t.csv.meta.md"), "t");
        assert_eq!(file_title("works/0001/.md"), ".md");

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

    #[test]
    fn acts() {
        let ch = |v: &[(char, &str)]| -> Vec<(char, String)> {
            v.iter().map(|(c, p)| (*c, p.to_string())).collect()
        };
        assert_eq!(infer_act(Some("adopt"), &[]), Act::Adopt);
        assert_eq!(infer_act(Some("nonsense"), &[]), Act::Update);
        assert_eq!(
            infer_act(
                None,
                &ch(&[('A', "problems/0001-x.md"), ('A', "works/0001/a.md")])
            ),
            Act::OpenProblem
        );
        assert_eq!(
            infer_act(None, &ch(&[('M', "works/0001/a.md")])),
            Act::UpdateWork
        );
        assert_eq!(infer_act(None, &ch(&[('A', "board/n.md")])), Act::Notice);
        assert_eq!(
            infer_act(None, &ch(&[('A', "talk/0001/20261018T212005Z-wang.md")])),
            Act::Note
        );
        assert_eq!(
            infer_act(None, &ch(&[('M', ".teahouse/members.toml")])),
            Act::Seat
        );
        assert_eq!(
            infer_act(None, &ch(&[('M', "problems/0001-x.md")])),
            Act::Update
        );
    }

    #[test]
    fn trailers() {
        assert_eq!(
            strip_trailers("理由一\n\n理由二\n\nProblem: 0001\nAct: decline\n"),
            "理由一\n\n理由二"
        );
        assert_eq!(strip_trailers("Problem: 0001"), "");
        assert_eq!(
            strip_trailers("就这样: 不行吗\n还有一行"),
            "就这样: 不行吗\n还有一行"
        );
    }
}
