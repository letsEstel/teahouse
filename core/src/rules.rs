//! Pure authorisation rules: which role may touch which path and which ref.
//! The relay feeds facts from git into these functions; nothing here does I/O,
//! so every rule is unit-testable and reusable by the desktop client to warn
//! users *before* they push.

use crate::members::{valid_member_id, Role};

/// Directories the steward may write on `main` (说明书 第四节 + 第十四节).
pub const STEWARD_DIRS: &[&str] = &["problems/", "works/", "board/", "talk/"];

/// Where written discussion lives (说明书 第十四节):
/// * `talk/<problem-id>/<stamp>-<member-id>.md` — under a problem
/// * `talk/board/<notice>/<stamp>-<member-id>.md` — replies to a notice
///
/// `<stamp>` is UTC `YYYYMMDDTHHMMSSZ`. One post, one file, one author.
/// The stamp only names the file and keeps names apart; posts are *shown* in
/// the order they reached `main`, so a stamp cannot reorder a discussion.
pub const TALK_DIR: &str = "talk/";
/// Where notices live: `board/<notice>.md`.
pub const BOARD_DIR: &str = "board/";

/// Largest post or notice: 32 KiB, roughly ten thousand Chinese characters.
/// Pictures and attachments belong in `works/`.
pub const MAX_TEXT_BYTES: usize = 32 * 1024;
/// How far a post's stamp may run ahead of the relay's clock. Earlier stamps
/// are fine: a post written offline is pushed later.
pub const MAX_STAMP_AHEAD_SECS: i64 = 10 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TalkTopic<'a> {
    Problem(&'a str),
    Board(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TalkPost<'a> {
    pub topic: TalkTopic<'a>,
    pub stamp: &'a str,
    pub author: &'a str,
}

/// Seconds since the Unix epoch of a `YYYYMMDDTHHMMSSZ` stamp; `None` unless
/// it is a real UTC date and time (no 2月30日, no 25点).
pub fn stamp_epoch(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if !(b.len() == 16
        && b[..8].iter().all(u8::is_ascii_digit)
        && b[8] == b'T'
        && b[9..15].iter().all(u8::is_ascii_digit)
        && b[15] == b'Z')
    {
        return None;
    }
    let num = |r: std::ops::Range<usize>| s[r].parse::<i64>().ok();
    let (y, mo, d) = (num(0..4)?, num(4..6)?, num(6..8)?);
    let (h, mi, se) = (num(9..11)?, num(11..13)?, num(13..15)?);
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_len = match mo {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    if !(1..=month_len).contains(&d) || h > 23 || mi > 59 || se > 59 {
        return None;
    }
    Some(days_from_civil(y, mo, d) * 86_400 + h * 3_600 + mi * 60 + se)
}

/// The `YYYYMMDDTHHMMSSZ` stamp for a moment, for naming a new post.
pub fn format_stamp(epoch: i64) -> String {
    let (days, secs) = (epoch.div_euclid(86_400), epoch.rem_euclid(86_400));
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}{m:02}{d:02}T{:02}{:02}{:02}Z",
        secs / 3_600,
        secs % 3_600 / 60,
        secs % 60
    )
}

/// Howard Hinnant's days→civil algorithm (proleptic Gregorian).
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// Howard Hinnant's civil→days algorithm (proleptic Gregorian).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * ((m + 9) % 12) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Posts and notices are plain text: at most [`MAX_TEXT_BYTES`], UTF-8, and
/// no control characters except tab and line breaks.
pub fn check_text(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > MAX_TEXT_BYTES {
        return Err(format!(
            "{} bytes; posts and notices are limited to 32 KiB ({MAX_TEXT_BYTES} bytes), put attachments in works/ / 留言和公告最多 32 KiB，附件请放进 works/",
            bytes.len()
        ));
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Err("is not UTF-8 text; posts and notices must be text, put attachments in works/ / 留言和公告必须是文本".into());
    };
    if let Some(c) = text
        .chars()
        .find(|c| c.is_control() && !matches!(c, '\t' | '\n' | '\r'))
    {
        return Err(format!(
            "contains the control character U+{:04X}; posts and notices must be plain text / 留言和公告必须是纯文本",
            c as u32
        ));
    }
    Ok(())
}

fn valid_notice_stem(s: &str) -> bool {
    (1..=80).contains(&s.len())
        && !s.starts_with('.')
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
}

/// Parses a discussion post path; `None` if it is not one.
pub fn parse_talk_path(path: &str) -> Option<TalkPost<'_>> {
    let rest = path.strip_prefix(TALK_DIR)?;
    let parts: Vec<&str> = rest.split('/').collect();
    let (topic, file) = match parts.as_slice() {
        ["board", notice, file] if valid_notice_stem(notice) => (TalkTopic::Board(notice), *file),
        [id, file] if id.len() == 4 && id.bytes().all(|b| b.is_ascii_digit()) => {
            (TalkTopic::Problem(id), *file)
        }
        _ => return None,
    };
    let stem = file.strip_suffix(".md")?;
    let (stamp, author) = stem.split_once('-')?;
    if stamp_epoch(stamp).is_none() || !valid_member_id(author) {
        return None;
    }
    Some(TalkPost {
        topic,
        stamp,
        author,
    })
}

pub const MAIN_REF: &str = "refs/heads/main";
pub const PROPOSALS_PREFIX: &str = "refs/proposals/";

/// May `role` change `path` in a commit landing on `main`?
pub fn may_write_on_main(role: Role, path: &str) -> bool {
    match role {
        Role::Host => true,
        Role::Steward => STEWARD_DIRS
            .iter()
            .any(|d| path.starts_with(d) && path.len() > d.len()),
        Role::Guest => false,
    }
}

/// Paths that would *become* `.teahouse/…` on a case-insensitive or
/// Windows file system (`.TeaHouse/`, `.teahouse./`, `.teahouse /`) without
/// being it in git. On a member's Windows or macOS checkout such a file could
/// overwrite the real seating chart, so it is refused for everyone.
pub fn is_reserved_lookalike(path: &str) -> bool {
    let first = path.split('/').next().unwrap_or("");
    if first == ".teahouse" {
        return false;
    }
    let normalised = first.trim_end_matches(['.', ' ']).to_ascii_lowercase();
    normalised == ".teahouse" || normalised.starts_with(".teahouse:") || normalised == "teahou~1"
}

/// A ref a client tried to update, classified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefKind<'a> {
    Main,
    /// `refs/proposals/<owner>/<name>`
    Proposal {
        owner: &'a str,
        name: &'a str,
    },
    Other,
}

pub fn classify_ref(r: &str) -> RefKind<'_> {
    if r == MAIN_REF {
        return RefKind::Main;
    }
    if let Some(rest) = r.strip_prefix(PROPOSALS_PREFIX) {
        if let Some((owner, name)) = rest.split_once('/') {
            if valid_member_id(owner) && valid_proposal_name(name) {
                return RefKind::Proposal { owner, name };
            }
        }
    }
    RefKind::Other
}

/// Proposal names: ASCII letters, digits, `-`, `_`, `.`; 1–64 chars; no `..`,
/// no leading dot, no `.lock` suffix (git's own restrictions, made stricter).
pub fn valid_proposal_name(n: &str) -> bool {
    (1..=64).contains(&n.len())
        && !n.starts_with('.')
        && !n.contains("..")
        && !n.ends_with(".lock")
        && n.bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_' || c == b'.')
}

/// The `Problem:` trailer every commit must carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProblemRef {
    /// `Problem: 0001`
    Id(String),
    /// `Problem: meta` — changes to the teahouse itself (host only).
    Meta,
    /// `Problem: board` — notices and replies; may touch only `board/` and
    /// `talk/board/`.
    Board,
}

impl ProblemRef {
    pub fn parse(v: &str) -> Option<ProblemRef> {
        let v = v.trim();
        if v == "meta" {
            return Some(ProblemRef::Meta);
        }
        if v == "board" {
            return Some(ProblemRef::Board);
        }
        if v.len() == 4 && v.bytes().all(|b| b.is_ascii_digit()) {
            return Some(ProblemRef::Id(v.to_string()));
        }
        None
    }

    /// Does a file name under `problems/` belong to this problem?
    /// Accepts `0001.md` and `0001-<slug>.md`.
    pub fn matches_problem_file(id: &str, file_name: &str) -> bool {
        let Some(rest) = file_name.strip_prefix(id) else {
            return false;
        };
        rest == ".md" || (rest.starts_with('-') && rest.ends_with(".md") && rest.len() > 4)
    }
}

/// What a member may do to a ref, before looking at the commits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefDecision {
    Allow,
    Deny(String),
}

/// `create`, `update` or `delete` — derived from the old/new object ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Create,
    Update,
    Delete,
}

pub fn decide_ref(role: Role, member_id: &str, kind: &RefKind<'_>, op: Op) -> RefDecision {
    use RefDecision::*;
    match kind {
        RefKind::Main => match (op, role) {
            (Op::Delete, _) => Deny("main can never be deleted / main 分支不能删除".into()),
            (Op::Create, _) => Deny("main already exists on the relay / main 只能由 relay 创建".into()),
            // Guests may update main only with their own talk/ posts; that is
            // judged commit by commit (see the relay's pre-receive hook).
            (Op::Update, _) => Allow,
        },
        RefKind::Proposal { owner, .. } => {
            // Your own namespace is yours; the host may also clear others'
            // proposals after deciding on them.
            if *owner == member_id || (op == Op::Delete && role == Role::Host) {
                Allow
            } else {
                Deny(format!(
                    "refs/proposals/{owner}/… belongs to {owner}; yours is refs/proposals/{member_id}/… / 只能写自己的提案分支"
                ))
            }
        }
        RefKind::Other => Deny(
            "only refs/heads/main and refs/proposals/<id>/<name> exist in a teahouse / 茶馆只有 main 和提案分支".into(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths() {
        assert!(may_write_on_main(Role::Host, ".teahouse/members.toml"));
        assert!(may_write_on_main(Role::Steward, "works/0001/plan.md"));
        assert!(may_write_on_main(Role::Steward, "problems/0002-smell.md"));
        assert!(may_write_on_main(Role::Steward, "board/2026-10-18.md"));
        assert!(!may_write_on_main(Role::Steward, ".teahouse/members.toml"));
        assert!(!may_write_on_main(Role::Steward, "README.md"));
        assert!(!may_write_on_main(Role::Steward, "works"));
        assert!(!may_write_on_main(Role::Steward, "worksX/a"));
        assert!(!may_write_on_main(Role::Guest, "works/0001/a.md"));
    }

    #[test]
    fn talk_paths() {
        assert_eq!(
            parse_talk_path("talk/0001/20261018T213000Z-wang.md"),
            Some(TalkPost {
                topic: TalkTopic::Problem("0001"),
                stamp: "20261018T213000Z",
                author: "wang"
            })
        );
        assert_eq!(
            parse_talk_path("talk/board/2026-10-18-meeting/20261018T213000Z-li_2.md"),
            Some(TalkPost {
                topic: TalkTopic::Board("2026-10-18-meeting"),
                stamp: "20261018T213000Z",
                author: "li_2"
            })
        );
        for bad in [
            "talk/0001/x.md",
            "talk/0001/20261018T213000Z-wang.txt",
            "talk/0001/20261018T213000-wang.md",
            "talk/0001/20261318T213000Z-wang.md",
            "talk/0001/20260230T213000Z-wang.md",
            "talk/0001/20261018T243000Z-wang.md",
            "talk/0001/20261018T213000Z-Wang.md",
            "talk/01/20261018T213000Z-wang.md",
            "talk/0001/sub/20261018T213000Z-wang.md",
            "talk/board/.x/20261018T213000Z-wang.md",
            "talk/board/20261018T213000Z-wang.md",
            "works/0001/20261018T213000Z-wang.md",
        ] {
            assert_eq!(parse_talk_path(bad), None, "{bad}");
        }
        assert!(may_write_on_main(
            Role::Steward,
            "talk/0001/20261018T213000Z-wang.md"
        ));
        assert_eq!(ProblemRef::parse("board"), Some(ProblemRef::Board));
    }

    #[test]
    fn stamps() {
        assert_eq!(stamp_epoch("19700101T000000Z"), Some(0));
        // Same instant as relay/src/audit.rs's rfc3339 test.
        assert_eq!(stamp_epoch("20261018T212005Z"), Some(1_792_358_405));
        assert_eq!(stamp_epoch("20000229T000000Z"), Some(951_782_400));
        assert_eq!(stamp_epoch("19991231T235959Z"), Some(946_684_799));
        for t in [0, 951_782_400, 1_792_358_405, 4_102_444_799, 1_234_567_890] {
            assert_eq!(stamp_epoch(&format_stamp(t)), Some(t), "{t}");
        }
        assert_eq!(format_stamp(1_792_358_405), "20261018T212005Z");
        for bad in [
            "20260229T000000Z", // 2026 is not a leap year
            "21000229T000000Z", // nor is 2100
            "20261000T000000Z",
            "20261032T000000Z",
            "20261018T236000Z",
            "20261018T235960Z",
            "20261018T213000",
            "2026-10-18T2130Z",
            "+0261018T213000Z",
        ] {
            assert_eq!(stamp_epoch(bad), None, "{bad}");
        }
    }

    #[test]
    fn post_text() {
        assert!(check_text("晚上七点以后基本没位置。\r\n\t第二行\n".as_bytes()).is_ok());
        assert!(check_text(&vec![b'a'; MAX_TEXT_BYTES]).is_ok());
        assert!(check_text(&vec![b'a'; MAX_TEXT_BYTES + 1])
            .unwrap_err()
            .contains("32 KiB"));
        // 10 922 three-byte characters fit; one more does not.
        assert!(check_text("茶".repeat(10_922).as_bytes()).is_ok());
        assert!(check_text("茶".repeat(10_923).as_bytes()).is_err());
        assert!(check_text(b"\x89PNG\r\n\x1a\n")
            .unwrap_err()
            .contains("UTF-8"));
        assert!(check_text(b"a\0b").unwrap_err().contains("U+0000"));
        assert!(check_text("\u{1b}[2J".as_bytes())
            .unwrap_err()
            .contains("U+001B"));
        assert!(check_text("\u{7f}".as_bytes()).is_err());
    }

    #[test]
    fn lookalikes() {
        for bad in [
            ".TeaHouse/members.toml",
            ".teahouse./members.toml",
            ".teahouse /x",
            ".TEAHOUSE",
            ".teahouse::$DATA/x",
            "TEAHOU~1/members.toml",
        ] {
            assert!(is_reserved_lookalike(bad), "{bad}");
        }
        for ok in [
            ".teahouse/members.toml",
            "works/.teahouse/x",
            "teahouse/x",
            ".teahousex/y",
        ] {
            assert!(!is_reserved_lookalike(ok), "{ok}");
        }
    }

    #[test]
    fn refs() {
        assert_eq!(classify_ref("refs/heads/main"), RefKind::Main);
        assert_eq!(
            classify_ref("refs/proposals/wang/night-share"),
            RefKind::Proposal {
                owner: "wang",
                name: "night-share"
            }
        );
        for other in [
            "refs/heads/dev",
            "refs/tags/v1",
            "refs/proposals/wang",
            "refs/proposals/wang/a/b",
            "refs/proposals/Wang/x",
            "refs/proposals/wang/..x",
            "refs/proposals/wang/x.lock",
            "refs/proposals/wang/.hidden",
        ] {
            assert_eq!(classify_ref(other), RefKind::Other, "{other}");
        }
    }

    #[test]
    fn decisions() {
        let p = classify_ref("refs/proposals/wang/x");
        assert_eq!(
            decide_ref(Role::Guest, "wang", &p, Op::Create),
            RefDecision::Allow
        );
        assert_eq!(
            decide_ref(Role::Guest, "wang", &p, Op::Delete),
            RefDecision::Allow
        );
        assert!(matches!(
            decide_ref(Role::Guest, "li", &p, Op::Create),
            RefDecision::Deny(_)
        ));
        assert!(matches!(
            decide_ref(Role::Steward, "zhou", &p, Op::Delete),
            RefDecision::Deny(_)
        ));
        assert_eq!(
            decide_ref(Role::Host, "chen", &p, Op::Delete),
            RefDecision::Allow
        );
        assert!(matches!(
            decide_ref(Role::Host, "chen", &p, Op::Update),
            RefDecision::Deny(_)
        ));

        let m = RefKind::Main;
        assert_eq!(
            decide_ref(Role::Guest, "wang", &m, Op::Update),
            RefDecision::Allow
        );
        assert_eq!(
            decide_ref(Role::Steward, "zhou", &m, Op::Update),
            RefDecision::Allow
        );
        assert!(matches!(
            decide_ref(Role::Host, "chen", &m, Op::Delete),
            RefDecision::Deny(_)
        ));
        assert!(matches!(
            decide_ref(Role::Host, "chen", &RefKind::Other, Op::Create),
            RefDecision::Deny(_)
        ));
    }

    #[test]
    fn problem_refs() {
        assert_eq!(
            ProblemRef::parse("0001"),
            Some(ProblemRef::Id("0001".into()))
        );
        assert_eq!(ProblemRef::parse(" meta "), Some(ProblemRef::Meta));
        for bad in ["1", "00001", "abcd", "META", ""] {
            assert_eq!(ProblemRef::parse(bad), None, "{bad}");
        }
        assert!(ProblemRef::matches_problem_file("0001", "0001.md"));
        assert!(ProblemRef::matches_problem_file("0001", "0001-停车位.md"));
        assert!(!ProblemRef::matches_problem_file("0001", "00012.md"));
        assert!(!ProblemRef::matches_problem_file("0001", "0001-.md"));
        assert!(!ProblemRef::matches_problem_file("0001", "0001-x.txt"));
    }
}
