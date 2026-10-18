//! `pre-receive` and `post-receive`: where the teahouse's laws are enforced.
//!
//! git runs `pre-receive` after it has received a push but before any ref
//! moves; if we exit non-zero, *nothing* in the push is applied. New objects
//! are still in git's quarantine, which we read through the environment git
//! hands us. Authority always comes from the seating chart on the **current**
//! main — never from the commits being pushed — so a push cannot promote the
//! person pushing it.

use crate::audit::{self, Entry};
use crate::bail;
use crate::error::{Error, Result};
use crate::git::{is_zero_oid, Git};
use crate::root::{members_at, Root};
use std::io::{self, BufRead, Read};
use teahouse_core::members::{Member, Members, Role, MEMBERS_PATH};
use teahouse_core::rules::{
    classify_ref, decide_ref, is_reserved_lookalike, may_write_on_main, parse_talk_path, Op,
    ProblemRef, RefDecision, RefKind, TalkTopic, MAIN_REF, TALK_DIR,
};
use teahouse_core::PublicKey;

/// Upper bound on commits examined per pushed ref. Keeps a hostile push from
/// tying up the relay; legitimate teahouses never come close.
pub const MAX_COMMITS_PER_PUSH: usize = 5_000;
/// Upper bound on ref updates in one push.
pub const MAX_REF_UPDATES: usize = 64;

#[derive(Debug, Clone)]
pub struct Update {
    pub old: String,
    pub new: String,
    pub refname: String,
}

pub fn read_updates(input: impl Read) -> Result<Vec<Update>> {
    let mut v = Vec::new();
    for line in io::BufReader::new(input).lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let mut it = line.splitn(3, ' ');
        let (Some(old), Some(new), Some(r)) = (it.next(), it.next(), it.next()) else {
            bail!("malformed hook input line {line:?}");
        };
        v.push(Update {
            old: old.into(),
            new: new.into(),
            refname: r.into(),
        });
    }
    Ok(v)
}

/// The facts a decision is made from.
pub struct Context<'a> {
    pub git: &'a Git,
    pub pusher: &'a Member,
}

fn op_of(u: &Update) -> Op {
    match (is_zero_oid(&u.old), is_zero_oid(&u.new)) {
        (true, _) => Op::Create,
        (_, true) => Op::Delete,
        _ => Op::Update,
    }
}

fn short(oid: &str) -> &str {
    &oid[..oid.len().min(10)]
}

/// Checks one ref update. Returns every reason it must be refused.
pub fn check_update(cx: &Context<'_>, u: &Update) -> Result<Vec<String>> {
    let kind = classify_ref(&u.refname);
    let op = op_of(u);
    let role = cx.pusher.role;
    if let RefDecision::Deny(why) = decide_ref(role, &cx.pusher.id, &kind, op) {
        return Ok(vec![format!("{}: {why}", u.refname)]);
    }
    if op == Op::Delete {
        return Ok(Vec::new());
    }

    let mut problems = Vec::new();
    let commits = match kind {
        RefKind::Main => {
            if !cx.git.is_ancestor(&u.old, &u.new)? {
                return Ok(vec![format!(
                    "{}: history can only grow; this push would rewrite it (force push refused) / 历史只能追加，不能改写",
                    u.refname
                )]);
            }
            let range = format!("{}..{}", u.old, u.new);
            cx.git.rev_list(&["--topo-order", "--reverse", &range])?
        }
        RefKind::Proposal { .. } => {
            cx.git
                .rev_list(&["--topo-order", "--reverse", &u.new, "--not", "--all"])?
        }
        RefKind::Other => unreachable!("decide_ref denies other refs"),
    };
    if commits.len() > MAX_COMMITS_PER_PUSH {
        return Ok(vec![format!(
            "{}: {} new commits in one push; the limit is {MAX_COMMITS_PER_PUSH}",
            u.refname,
            commits.len()
        )]);
    }

    let on_main = matches!(kind, RefKind::Main);
    for c in &commits {
        check_commit(cx, c, on_main, &mut problems)?;
        if problems.len() >= 20 {
            problems.push("… (more problems omitted)".into());
            break;
        }
    }
    Ok(problems
        .into_iter()
        .map(|p| format!("{}: {p}", u.refname))
        .collect())
}

fn check_commit(cx: &Context<'_>, c: &str, on_main: bool, out: &mut Vec<String>) -> Result<()> {
    let role = cx.pusher.role;
    let me = cx.pusher.id.as_str();
    let sc = short(c);

    // 1. Every commit faces exactly one problem (说明书 第三节).
    let values = cx.git.trailer_values(c, "Problem")?;
    let facing = match values.as_slice() {
        [] => {
            out.push(format!(
                "commit {sc} has no `Problem:` trailer; every commit must face a problem (e.g. `Problem: 0001`) / 每个提交都必须面向一个问题"
            ));
            None
        }
        [one] => match ProblemRef::parse(one) {
            None => {
                out.push(format!(
                    "commit {sc}: `Problem: {one}` is not a 4-digit problem id, `board` or `meta`"
                ));
                None
            }
            Some(p) => Some(p),
        },
        _ => {
            out.push(format!(
                "commit {sc} has {} `Problem:` trailers; exactly one is required",
                values.len()
            ));
            None
        }
    };
    match &facing {
        Some(ProblemRef::Meta) if on_main && role != Role::Host => out.push(format!(
            "commit {sc}: `Problem: meta` changes the teahouse itself; only the host may land it on main / 只有老板能提交 meta"
        )),
        Some(ProblemRef::Id(id)) => {
            let files = cx.git.list_dir(c, "problems")?;
            if !files.iter().any(|f| ProblemRef::matches_problem_file(id, f)) {
                out.push(format!(
                    "commit {sc}: problem {id} does not exist (no problems/{id}-*.md in this commit) / 问题 {id} 不存在"
                ));
            }
        }
        _ => {}
    }

    // 2. Guests land on main only by *adding* their own talk/ posts.
    if on_main && role == Role::Guest {
        if cx.git.parents(c)?.len() >= 2 {
            out.push(format!(
                "commit {sc}: guests cannot write main with a merge; push to refs/proposals/{me}/<name> instead / 茶客不能直接写入 main，请发起提案"
            ));
        } else {
            for (status, path) in cx.git.changes(c)? {
                let own_new_post =
                    status == 'A' && parse_talk_path(&path).is_some_and(|post| post.author == me);
                if !own_new_post {
                    out.push(format!(
                        "commit {sc}: guests cannot write main except adding their own posts under talk/ (not {path}); push to refs/proposals/{me}/<name> instead / 茶客只能在 talk/ 下新增自己的留言，其余请发起提案"
                    ));
                }
            }
        }
    }

    // 3. Paths: what this commit itself changes.
    let paths = cx.git.changed_paths(c)?;
    let mut members_changed = false;
    for p in &paths {
        if is_reserved_lookalike(p) {
            out.push(format!(
                "commit {sc}: {p} imitates .teahouse/ on Windows/macOS file systems / 路径冒充 .teahouse/"
            ));
            continue;
        }
        if p == MEMBERS_PATH {
            members_changed = true;
        } else if p.starts_with(".teahouse/") {
            out.push(format!(
                "commit {sc}: {p} — .teahouse/ holds only members.toml"
            ));
            continue;
        }
        if on_main && role != Role::Guest && !may_write_on_main(role, p) {
            out.push(format!(
                "commit {sc}: the steward may only change problems/, works/, board/ and talk/, not {p} / 伙计只能修改 problems/、works/、board/、talk/"
            ));
        }
        // A post must sit under the topic its commit faces.
        if let Some(post) = parse_talk_path(p) {
            let consistent = match (&post.topic, &facing) {
                (TalkTopic::Problem(id), Some(ProblemRef::Id(f))) => id == f,
                (TalkTopic::Board(_), Some(ProblemRef::Board)) => true,
                (_, Some(ProblemRef::Meta)) => role == Role::Host,
                _ => false,
            };
            if !consistent && facing.is_some() {
                out.push(format!(
                    "commit {sc}: {p} is filed under a different topic than the commit's `Problem:` trailer"
                ));
            }
        } else if on_main && role == Role::Steward && p.starts_with(TALK_DIR) {
            out.push(format!(
                "commit {sc}: {p} is not a valid post path (talk/<id>/<YYYYMMDDTHHMMSSZ>-<member>.md or talk/board/<notice>/…)"
            ));
        }
        if matches!(facing, Some(ProblemRef::Board))
            && !(p.starts_with("board/") || p.starts_with("talk/board/"))
        {
            out.push(format!(
                "commit {sc}: `Problem: board` may only change board/ and talk/board/, not {p}"
            ));
        }
    }

    // 4. A seating chart in history must always be a valid one.
    if members_changed {
        match cx.git.read_file(c, MEMBERS_PATH)? {
            None => out.push(format!(
                "commit {sc} deletes {MEMBERS_PATH}; a teahouse cannot exist without it"
            )),
            Some(bytes) => match std::str::from_utf8(&bytes) {
                Err(_) => out.push(format!("commit {sc}: {MEMBERS_PATH} is not UTF-8")),
                Ok(text) => {
                    if let Err(e) = Members::parse(text) {
                        out.push(format!("commit {sc}: {e}"));
                    }
                }
            },
        }
    }
    Ok(())
}

fn env(name: &str) -> Result<String> {
    std::env::var(name).map_err(|_| {
        Error(format!(
            "pushes must arrive through the relay (missing {name}) / 推送必须经过 relay"
        ))
    })
}

pub fn pre_receive(root: &Root) -> Result<()> {
    let key_b64 = env("TEAHOUSE_KEY")?;
    let house = env("TEAHOUSE_HOUSE")?;
    let key = PublicKey::from_blob_b64(&key_b64)?;
    let updates = read_updates(io::stdin().lock())?;
    let git = Git::inherited();

    let members = members_at(&git, MAIN_REF)?;
    let Some(pusher) = members.by_key(&key) else {
        audit::log(
            root,
            Entry::new("push.denied")
                .house(&house)
                .key(&key.short())
                .field("reason", "seat removed"),
        );
        bail!("you no longer have a seat in this teahouse / 你在这间茶馆已没有座位");
    };

    let mut problems = Vec::new();
    if updates.len() > MAX_REF_UPDATES {
        problems.push(format!(
            "{} ref updates in one push; the limit is {MAX_REF_UPDATES}",
            updates.len()
        ));
    } else {
        let cx = Context { git: &git, pusher };
        for u in &updates {
            problems.extend(check_update(&cx, u)?);
        }
    }

    let refs: Vec<String> = updates
        .iter()
        .map(|u| format!("{} {}..{}", u.refname, short(&u.old), short(&u.new)))
        .collect();

    if problems.is_empty() {
        audit::log(
            root,
            Entry::new("push.allowed")
                .house(&house)
                .member(&pusher.id)
                .key(&key.short())
                .field("role", pusher.role.as_str())
                .field("refs", refs.join(", ")),
        );
        return Ok(());
    }

    audit::log(
        root,
        Entry::new("push.denied")
            .house(&house)
            .member(&pusher.id)
            .key(&key.short())
            .field("role", pusher.role.as_str())
            .field("refs", refs.join(", "))
            .field("reasons", problems.join(" | ")),
    );
    eprintln!();
    eprintln!(
        "茶馆拒绝了这次推送 · The teahouse refused this push ({} as {} / {}):",
        pusher.id,
        pusher.role.zh(),
        pusher.role
    );
    for p in &problems {
        eprintln!("  ✗ {p}");
    }
    eprintln!();
    Err(Error("push refused".into()))
}

pub fn post_receive(root: &Root) -> Result<()> {
    let house = std::env::var("TEAHOUSE_HOUSE").unwrap_or_default();
    let updates = read_updates(io::stdin().lock())?;
    let git = Git::inherited();
    let mut seats_changed = false;
    for u in &updates {
        if u.refname == MAIN_REF && !is_zero_oid(&u.new) && !is_zero_oid(&u.old) {
            let out = git.run(&[
                "diff",
                "--name-only",
                "--no-renames",
                &u.old,
                &u.new,
                "--",
                MEMBERS_PATH,
            ])?;
            if !out.is_empty() {
                seats_changed = true;
            }
        }
    }
    audit::log(
        root,
        Entry::new("push.applied").house(&house).field(
            "refs",
            updates
                .iter()
                .map(|u| format!("{} {}", u.refname, short(&u.new)))
                .collect::<Vec<_>>()
                .join(", "),
        ),
    );
    if seats_changed {
        audit::log(root, Entry::new("seats.changed").house(&house));
        let cfg = root.load_config()?;
        if let Some(path) = cfg.authorized_keys {
            if let Err(e) = crate::admin::sync_keys(root, &cfg.binary, &path) {
                eprintln!("teahouse-relay: warning: could not refresh {path}: {e}");
            }
        }
    }
    Ok(())
}
