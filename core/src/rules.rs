//! Pure authorisation rules: which role may touch which path and which ref.
//! The relay feeds facts from git into these functions; nothing here does I/O,
//! so every rule is unit-testable and reusable by the desktop client to warn
//! users *before* they push.

use crate::members::{valid_member_id, Role};

/// Directories the steward may write on `main` (说明书 第四节 + 第十四节).
pub const STEWARD_DIRS: &[&str] = &["problems/", "works/", "board/"];

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
}

impl ProblemRef {
    pub fn parse(v: &str) -> Option<ProblemRef> {
        let v = v.trim();
        if v == "meta" {
            return Some(ProblemRef::Meta);
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
            (Op::Update, Role::Guest) => Deny(
                "guests cannot write main; push to refs/proposals/<your-id>/<name> instead / 茶客不能直接写入 main，请推送到提案分支".into(),
            ),
            (Op::Update, Role::Host | Role::Steward) => Allow,
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
        assert!(matches!(
            decide_ref(Role::Guest, "wang", &m, Op::Update),
            RefDecision::Deny(_)
        ));
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
