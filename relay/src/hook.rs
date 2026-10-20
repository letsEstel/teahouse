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
use std::path::Path;
use teahouse_core::members::{Member, Members, Role, MEMBERS_PATH};
use teahouse_core::rules::{
    check_text, classify_ref, decide_ref, is_reserved_lookalike, may_write_on_main,
    parse_talk_path, stamp_epoch, Op, ProblemRef, RefDecision, RefKind, TalkTopic, BOARD_DIR,
    MAIN_REF, MAX_STAMP_AHEAD_SECS, MAX_TEXT_BYTES, TALK_DIR,
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
    /// The relay's clock when the push arrived, in Unix seconds.
    pub now: i64,
    /// This teahouse's attachment store (`<root>/blobs/<id>/`).
    pub blobs: &'a Path,
    /// An allowed-signers file of every key ever seated here; `None` for
    /// commits the relay writes itself (not pushed, not member-signed).
    pub signers: Option<&'a Path>,
}

/// Files larger than this must not be binary in git (§1.6: attachments go
/// to the attachment store; only a reference goes into history).
pub const MAX_BINARY_IN_GIT: u64 = 1024 * 1024;

/// An allowed-signers file (`ssh-keygen -Y`) naming every key, of every
/// device, on the seating chart at `rev` (the chart as the push finds it),
/// as `<id>@teahouse`. 法第 13 条 as tightened (docs/DEVICES.md §7): a
/// newly pushed commit counts only if signed by a key seated **now**, so a
/// removed device stops counting at once.
pub fn write_allowed_signers(git: &Git, rev: &str, path: &Path) -> Result<()> {
    let mut lines = std::collections::BTreeSet::new();
    for x in members_at(git, rev)?.members {
        for k in x.keys() {
            lines.insert(format!(
                "{}@teahouse namespaces=\"git\" {}",
                x.id,
                k.openssh()
            ));
        }
    }
    let mut text = String::new();
    for l in lines {
        text.push_str(&l);
        text.push('\n');
    }
    std::fs::write(path, text)?;
    Ok(())
}

/// Law: every pushed commit is signed with the device key of the member it
/// names as its author (`<id>@teahouse`). git checks the signatures (with
/// ssh-keygen), in one run for all commits of the update.
fn check_signatures(cx: &Context<'_>, commits: &[String], out: &mut Vec<String>) -> Result<()> {
    let Some(signers) = cx.signers else {
        return Ok(());
    };
    if commits.is_empty() {
        return Ok(());
    }
    let file = format!("gpg.ssh.allowedSignersFile={}", signers.display());
    let mut input = commits.join("\n");
    input.push('\n');
    let out_bytes = cx.git.run_input(
        &[
            "-c",
            "gpg.format=ssh",
            "-c",
            &file,
            "log",
            "--no-walk=unsorted",
            "--stdin",
            "--format=%H%x00%G?%x00%GS%x00%ae",
        ],
        Some(input.as_bytes()),
    )?;
    for line in String::from_utf8_lossy(&out_bytes).lines() {
        let f: Vec<&str> = line.split('\0').collect();
        let [hash, status, signer, author] = f.as_slice() else {
            continue;
        };
        let sc = short(hash);
        let me = format!("{}@teahouse", cx.pusher.id);
        match *status {
            "G" if signer == author => {}
            "G" => out.push(format!(
                "commit {sc} names {author} as its author but is signed by {signer} / 作者和签名不是同一个人"
            )),
            "N" => out.push(format!(
                "commit {sc} is not signed; every commit must be signed with its author's device key (git: gpg.format=ssh, commit.gpgsign=true, user.email={me}) / 每个提交都必须用作者的设备钥匙签名"
            )),
            "U" | "E" => out.push(format!(
                "commit {sc} is signed with a key that has no seat here now (a removed device, or a member who left) / 签名的钥匙现在不在座位表上"
            )),
            other => out.push(format!(
                "commit {sc}: its signature does not verify ({other}) / 签名无效"
            )),
        }
        if out.len() >= 20 {
            break;
        }
    }
    Ok(())
}

/// Law: on `main`, one problem file per number.
fn check_unique_numbers(cx: &Context<'_>, c: &str, out: &mut Vec<String>) -> Result<()> {
    let mut seen: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for f in cx.git.list_dir(c, "problems")? {
        let id = f.get(..4).filter(|id| {
            id.bytes().all(|b| b.is_ascii_digit()) && ProblemRef::matches_problem_file(id, &f)
        });
        if let Some(id) = id {
            seen.entry(id.to_string()).or_default().push(f.clone());
        }
    }
    for (id, files) in seen {
        if files.len() > 1 {
            out.push(format!(
                "commit {}: problem {id} would have {} files ({}); each number names one problem / 同一个编号只能有一个问题",
                short(c),
                files.len(),
                files.join(", ")
            ));
        }
    }
    Ok(())
}

/// Law: attachments stay out of git. A `*.ref.md` reference must be well
/// formed and, unless purged, point at a file in this teahouse's attachment
/// store; any other file over 1 MiB must not be binary.
fn check_attachment(cx: &Context<'_>, c: &str, p: &str, out: &mut Vec<String>) -> Result<()> {
    let sc = short(c);
    let Some(entry) = cx.git.tree_entry(c, p)? else {
        return Ok(()); // removed
    };
    if p.to_ascii_lowercase().ends_with(".ref.md") {
        if !entry.is_regular_file() || entry.size.unwrap_or(u64::MAX) > MAX_TEXT_BYTES as u64 {
            out.push(format!(
                "commit {sc}: {p} must be a small text file (an attachment reference)"
            ));
            return Ok(());
        }
        let text =
            String::from_utf8_lossy(&cx.git.read_file(c, p)?.unwrap_or_default()).into_owned();
        match teahouse_core::frontmatter::check_blob_ref(&text) {
            Err(why) => out.push(format!("commit {sc}: {p} {why}")),
            Ok(r) if r.purged => {}
            Ok(r) => match std::fs::metadata(cx.blobs.join(&r.hash)) {
                Ok(m) if m.len() == r.size => {}
                Ok(m) => out.push(format!(
                    "commit {sc}: {p} says {} bytes, but the stored attachment has {} / 附件大小不符",
                    r.size,
                    m.len()
                )),
                Err(_) => out.push(format!(
                    "commit {sc}: {p} refers to sha256:{} which is not in the attachment store; upload it first / 附件还没有上传",
                    r.hash
                )),
            },
        }
        return Ok(());
    }
    if entry.kind == "blob" && entry.size.unwrap_or(0) > MAX_BINARY_IN_GIT {
        let bytes = cx.git.read_file(c, p)?.unwrap_or_default();
        if bytes.iter().take(8000).any(|&b| b == 0) {
            out.push(format!(
                "commit {sc}: {p} is a binary file of {} bytes; files over 1 MiB that are not text go to the attachment store, with a *.ref.md in git / 超过 1 MiB 的二进制文件请作为附件上传",
                bytes.len()
            ));
        }
    }
    Ok(())
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
    // Signatures are checked on commits new to the relay. Commits it already
    // holds (a proposal being adopted, say) were checked when they arrived,
    // and history is not judged again by today's chart.
    let fresh = match kind {
        RefKind::Main => {
            cx.git
                .rev_list(&["--topo-order", "--reverse", &u.new, "--not", "--all"])?
        }
        _ => commits.clone(),
    };
    check_signatures(cx, &fresh, &mut problems)?;
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
                if let Some(why) = guest_change_refused(me, status, &path) {
                    out.push(format!("commit {sc}: {why}"));
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
        if p.starts_with(TALK_DIR) || p.starts_with(BOARD_DIR) {
            check_discussion_file(cx, c, p, &facing, out)?;
        }
        check_attachment(cx, c, p, out)?;
        if matches!(facing, Some(ProblemRef::Board))
            && !(p.starts_with("board/") || p.starts_with("talk/board/"))
        {
            out.push(format!(
                "commit {sc}: `Problem: board` may only change board/ and talk/board/, not {p}"
            ));
        }
    }

    if on_main {
        check_unique_numbers(cx, c, out)?;
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

/// Why a guest may not make this change on `main`, as precisely as we can
/// tell; `None` if it is their own new post.
fn guest_change_refused(me: &str, status: char, path: &str) -> Option<String> {
    let post = parse_talk_path(path);
    match (status, &post) {
        ('A', Some(p)) if p.author == me => None,
        ('A', Some(p)) => Some(format!(
            "{path} is signed by {}, not by you; name your post …-{me}.md / 只能以自己的名义留言（文件名结尾应是 -{me}.md）",
            p.author
        )),
        ('M', Some(_)) => Some(format!(
            "{path} was already posted; posts are never edited, add a new post instead / 留言写下后不能修改，请另发一条"
        )),
        ('D', Some(_)) => Some(format!(
            "{path}: guests cannot remove posts; ask the steward or host / 茶客不能删除留言，请找伙计或老板"
        )),
        ('A', None) if path.starts_with(TALK_DIR) => Some(format!(
            "{path} is not a post path; use talk/<problem-id>/<YYYYMMDDTHHMMSSZ>-{me}.md or talk/board/<notice>/<YYYYMMDDTHHMMSSZ>-{me}.md with a real UTC time / 留言路径不对"
        )),
        _ => Some(format!(
            "guests cannot write main except adding their own posts under talk/ (not {path}); push to refs/proposals/{me}/<name> instead / 茶客只能在 talk/ 下新增自己的留言，其余请发起提案"
        )),
    }
}

/// Posts and notices (说明书 第十四节): plain text of at most 32 KiB; a post
/// answers a problem or notice that exists, and its stamp is not ahead of
/// the relay's clock by more than ten minutes. Removed files are not checked.
fn check_discussion_file(
    cx: &Context<'_>,
    c: &str,
    p: &str,
    facing: &Option<ProblemRef>,
    out: &mut Vec<String>,
) -> Result<()> {
    let sc = short(c);
    let Some(entry) = cx.git.tree_entry(c, p)? else {
        return Ok(()); // removed (moderation)
    };
    if !entry.is_regular_file() {
        out.push(format!(
            "commit {sc}: {p} must be a plain text file, not a {} / 留言和公告必须是普通文本文件",
            if entry.kind == "blob" {
                "symlink"
            } else {
                &entry.kind
            }
        ));
        return Ok(());
    }
    let size = entry.size.unwrap_or(u64::MAX);
    if size > MAX_TEXT_BYTES as u64 {
        // Do not read it: the size alone decides.
        out.push(format!(
            "commit {sc}: {p} is {size} bytes; posts and notices are limited to 32 KiB ({MAX_TEXT_BYTES} bytes), put attachments in works/ / 留言和公告最多 32 KiB，附件请放进 works/"
        ));
    } else if let Some(bytes) = cx.git.read_file(c, p)? {
        if let Err(why) = check_text(&bytes) {
            out.push(format!("commit {sc}: {p} {why}"));
        }
    }

    let Some(post) = parse_talk_path(p) else {
        return Ok(());
    };
    if let Some(t) = stamp_epoch(post.stamp) {
        if t > cx.now + MAX_STAMP_AHEAD_SECS {
            out.push(format!(
                "commit {sc}: {p} is stamped {} minutes in the future; a stamp may be at most {} minutes ahead of the relay's clock / 留言时间不能晚于推送时间 10 分钟以上",
                (t - cx.now) / 60,
                MAX_STAMP_AHEAD_SECS / 60
            ));
        }
    }
    match post.topic {
        // Already reported through the commit's own `Problem:` trailer.
        TalkTopic::Problem(id) if matches!(facing, Some(ProblemRef::Id(f)) if f == id) => {}
        TalkTopic::Problem(id) => {
            let files = cx.git.list_dir(c, "problems")?;
            if !files
                .iter()
                .any(|f| ProblemRef::matches_problem_file(id, f))
            {
                out.push(format!(
                    "commit {sc}: {p} answers problem {id}, which does not exist (no problems/{id}-*.md) / 问题 {id} 不存在"
                ));
            }
        }
        TalkTopic::Board(notice) => {
            let file = format!("{BOARD_DIR}{notice}.md");
            if !cx
                .git
                .tree_entry(c, &file)?
                .is_some_and(|e| e.is_regular_file())
            {
                out.push(format!(
                    "commit {sc}: {p} replies to notice {notice:?}, which does not exist (no {file}) / 回复的公告不存在"
                ));
            }
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
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| Error("the relay's clock is before 1970".into()))?
            .as_secs() as i64;
        let signers = root
            .tmp_dir()
            .join(format!("allowed-signers-{}", std::process::id()));
        write_allowed_signers(&git, MAIN_REF, &signers)?;
        let blobs = root.blobs_dir(&house);
        let cx = Context {
            git: &git,
            pusher,
            now,
            blobs: &blobs,
            signers: Some(&signers),
        };
        let result = updates
            .iter()
            .map(|u| check_update(&cx, u))
            .collect::<Result<Vec<_>>>();
        let _ = std::fs::remove_file(&signers);
        problems.extend(result?.into_iter().flatten());
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
