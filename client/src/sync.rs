//! Synchronising a local clone with its relay (CLIENT-API.md §4 "同步").
//!
//! Remote-tracking refs remember what the relay had at the last sync:
//! `refs/remotes/origin/main` and `refs/remotes/origin/proposals/*`. A sync
//!
//! 1. fetches both;
//! 2. for every proposal ref, three-way: unchanged locally since the last
//!    sync → take the relay's; changed locally → push ours (or our deletion);
//! 3. for `main`: fast-forward when behind, push when ahead, and when both
//!    moved, replays our commits on top of the relay's `main` (re-signed),
//!    renumbering a new problem whose number someone else took meanwhile;
//! 4. pushes everything in one push, so the relay judges it as a whole.
//!    When someone pushed between our fetch and our push, the relay refuses
//!    ours as not a fast-forward: then the sync starts over from 1, on top
//!    of theirs (a few times), and only a refusal on the relay's own terms
//!    is `relay_refused`.

use crate::commit;
use crate::error::{Code, Error, Result};
use crate::format;
use crate::identity::Identity;
use crate::transport::{self, Report};
use git2::{
    build::TreeUpdateBuilder, Delta, FetchOptions, FetchPrune, Oid, PushOptions, Repository,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::rc::Rc;

pub const MAIN: &str = "refs/heads/main";
pub const TRACKING_MAIN: &str = "refs/remotes/origin/main";
pub const PROPOSALS: &str = "refs/proposals/";
pub const TRACKING_PROPOSALS: &str = "refs/remotes/origin/proposals/";
const RECORD: &str = "teahouse-sync.json";

/// What the last sync attempt left behind, kept in the git directory.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    /// When a sync last succeeded (seconds since the epoch).
    pub last_ok: Option<i64>,
    /// `offline` or `error` when the last attempt failed.
    pub failed: Option<String>,
    pub message: Option<String>,
}

impl Record {
    pub fn load(git_dir: &Path) -> Record {
        std::fs::read(git_dir.join(RECORD))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    fn save(&self, git_dir: &Path) {
        if let Ok(b) = serde_json::to_vec(self) {
            let _ = std::fs::write(git_dir.join(RECORD), b);
        }
    }
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// A connection-less handle on one clone and its relay.
pub struct Link<'a> {
    pub repo: &'a Repository,
    pub identity: &'a Identity,
    pub known_hosts: &'a Path,
    /// The engine's attachment cache (`<root>/blobs`).
    pub blobs: &'a Path,
}

/// Attachments the host purged while offline: removed from the relay's
/// store after the purge commit is pushed.
const PENDING_PURGES: &str = "teahouse-purges";

pub fn add_pending_purge(git_dir: &Path, hash: &str) -> Result<()> {
    let p = git_dir.join(PENDING_PURGES);
    let mut text = std::fs::read_to_string(&p).unwrap_or_default();
    if !text.lines().any(|l| l == hash) {
        text.push_str(hash);
        text.push('\n');
    }
    std::fs::write(p, text)?;
    Ok(())
}

pub fn remove_pending_purge(git_dir: &Path, hash: &str) -> Result<()> {
    let p = git_dir.join(PENDING_PURGES);
    let text = std::fs::read_to_string(&p).unwrap_or_default();
    let kept: String = text
        .lines()
        .filter(|l| *l != hash)
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(p, kept)?;
    Ok(())
}

/// The teahouse id in a teahouse URL (`ssh://…/<id>`, `teahouse-local://<id>/…`).
pub fn house_of_url(url: &str) -> Option<String> {
    if let Some(rest) = url.strip_prefix(&format!("{}://", transport::LOCAL_SCHEME)) {
        return rest.split('/').next().map(str::to_string);
    }
    url.trim_end_matches('/')
        .rsplit('/')
        .next()
        .map(|h| h.trim_end_matches(".git").to_string())
}

/// How many times a sync starts over when others push at the same moment.
const TRIES: usize = 3;

/// Why [`Link::sync_inner`] did not finish.
enum Retry {
    /// Someone pushed between our fetch and our push.
    Again,
    Stop(Error),
}
use Retry::{Again, Stop};

impl From<Error> for Retry {
    fn from(e: Error) -> Retry {
        Stop(e)
    }
}

impl From<git2::Error> for Retry {
    fn from(e: git2::Error) -> Retry {
        Stop(e.into())
    }
}

/// How a sync ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Everything local is on the relay and the other way round.
    Synced,
    /// There is no relay to sync with (a teahouse made locally).
    NoRemote,
}

fn refs_under(repo: &Repository, prefix: &str) -> Result<BTreeMap<String, Oid>> {
    let mut m = BTreeMap::new();
    for r in repo.references()? {
        let r = r?;
        if let (Some(name), Some(oid)) = (r.name().ok(), r.target()) {
            if let Some(rest) = name.strip_prefix(prefix) {
                m.insert(rest.to_string(), oid);
            }
        }
    }
    Ok(m)
}

fn set_ref(repo: &Repository, name: &str, to: Option<Oid>) -> Result<()> {
    match to {
        Some(o) => {
            repo.reference(name, o, true, "teahouse sync")?;
        }
        None => {
            if let Ok(mut r) = repo.find_reference(name) {
                r.delete()?;
            }
        }
    }
    Ok(())
}

impl Link<'_> {
    fn has_remote(&self) -> bool {
        self.repo.find_remote("origin").is_ok()
    }

    /// Fetches `main` and every proposal into the remote-tracking refs.
    pub fn fetch(&self) -> Result<()> {
        let mut remote = self.repo.find_remote("origin")?;
        let report = Rc::new(RefCell::new(Report::default()));
        let result = transport::with_key(self.identity, self.known_hosts, &report, || {
            let mut opts = FetchOptions::new();
            opts.remote_callbacks(transport::callbacks(report.clone()));
            opts.prune(FetchPrune::On);
            opts.download_tags(git2::AutotagOption::None);
            remote.fetch(
                &[
                    &format!("+{MAIN}:{TRACKING_MAIN}"),
                    &format!("+{PROPOSALS}*:{TRACKING_PROPOSALS}*"),
                ],
                Some(&mut opts),
                Some("teahouse sync"),
            )
        });
        result.map_err(|e| transport::classify(&e, &report.borrow()))
    }

    /// Pushes `specs` in one push. A refusal comes back as `relay_refused`
    /// with the relay's reasons.
    pub fn push(&self, specs: &[String]) -> Result<()> {
        if specs.is_empty() {
            return Ok(());
        }
        let mut remote = self.repo.find_remote("origin")?;
        let report = Rc::new(RefCell::new(Report::default()));
        let result = transport::with_key(self.identity, self.known_hosts, &report, || {
            let mut opts = PushOptions::new();
            opts.remote_callbacks(transport::callbacks(report.clone()));
            let specs: Vec<&str> = specs.iter().map(String::as_str).collect();
            remote.push(&specs, Some(&mut opts))
        });
        let r = report.borrow();
        if let Err(e) = result {
            // A hook refusal can surface as an error of the whole push.
            if !r.refused.is_empty() || e.class() == git2::ErrorClass::Reference {
                return Err(refused(&r));
            }
            return Err(transport::classify(&e, &r));
        }
        if !r.refused.is_empty() {
            return Err(refused(&r));
        }
        Ok(())
    }

    /// One full sync of this clone (see the module documentation).
    pub fn sync(&self) -> Result<Outcome> {
        let git_dir = self.repo.path().to_path_buf();
        if !self.has_remote() {
            return Ok(Outcome::NoRemote);
        }
        let mut record = Record::load(&git_dir);
        let mut result = self.sync_inner();
        for _ in 1..TRIES {
            match result {
                Err(Again) => result = self.sync_inner(),
                _ => break,
            }
        }
        let result = result.map_err(|e| match e {
            Again => Error::new(
                Code::RelayRefused,
                "others kept writing at the same time; try again / 别人也在同时写入，请再试一次",
            ),
            Stop(e) => e,
        });
        match &result {
            Ok(_) => {
                record.last_ok = Some(now());
                record.failed = None;
                record.message = None;
            }
            Err(e) => {
                record.failed = Some(
                    if e.code == Code::Offline {
                        "offline"
                    } else {
                        "error"
                    }
                    .into(),
                );
                // The relay's own words, when it gave a reason.
                record.message = Some(match e.reasons.first() {
                    Some(r) => format!("{}: {r}", e.message),
                    None => e.message.clone(),
                });
            }
        }
        record.save(&git_dir);
        result
    }

    fn sync_inner(&self) -> std::result::Result<Outcome, Retry> {
        let repo = self.repo;
        let old_main = repo.refname_to_id(TRACKING_MAIN).ok();
        let old_props = refs_under(repo, TRACKING_PROPOSALS)?;
        self.fetch()?;
        let new_main = repo.refname_to_id(TRACKING_MAIN).ok();
        let new_props = refs_under(repo, TRACKING_PROPOSALS)?;
        let local_props = refs_under(repo, PROPOSALS)?;

        let mut specs = Vec::new();
        let mut after: Vec<(String, Option<Oid>)> = Vec::new();
        let names: BTreeSet<&String> = old_props
            .keys()
            .chain(new_props.keys())
            .chain(local_props.keys())
            .collect();
        for name in names {
            let (loc, old, new) = (
                local_props.get(name).copied(),
                old_props.get(name).copied(),
                new_props.get(name).copied(),
            );
            let local_ref = format!("{PROPOSALS}{name}");
            if loc == old || loc == new {
                // Not changed here since the last sync (or already equal).
                set_ref(repo, &local_ref, new)?;
            } else if let (Some(l), Some(_)) = (loc, new.filter(|n| Some(*n) != old)) {
                // Changed here and, meanwhile, at the relay too: another of
                // my devices reworked the same proposal (docs/DEVICES.md §5).
                // Never force over it: this device's version becomes a
                // proposal of its own, and the relay's is kept as it is.
                let alt = format!("{name}-{}", crate::write::suffix());
                let alt_ref = format!("{PROPOSALS}{alt}");
                set_ref(repo, &alt_ref, Some(l))?;
                set_ref(repo, &local_ref, new)?;
                specs.push(format!("{alt_ref}:{alt_ref}"));
                after.push((format!("{TRACKING_PROPOSALS}{alt}"), Some(l)));
            } else {
                specs.push(match loc {
                    Some(_) => format!("+{local_ref}:{local_ref}"),
                    None => format!(":{local_ref}"),
                });
                after.push((format!("{TRACKING_PROPOSALS}{name}"), loc));
            }
        }

        let local_main = repo.refname_to_id(MAIN).ok();
        let _ = old_main;
        match (local_main, new_main) {
            (_, None) => {
                return Err(Error::internal("the relay has no main for this teahouse").into());
            }
            (None, Some(t)) => set_ref(repo, MAIN, Some(t))?,
            (Some(l), Some(t)) if l == t => {}
            (Some(l), Some(t)) if repo.graph_descendant_of(t, l)? => {
                commit::move_ref(repo, MAIN, Some(l), Some(t), "teahouse sync: fast-forward")?;
            }
            (Some(l), Some(t)) => {
                let tip = if repo.graph_descendant_of(l, t)? {
                    l
                } else {
                    let base = repo.merge_base(l, t)?;
                    let ours = first_parent_since(repo, l, base)?;
                    let (tip, _) = replay(repo, self.identity, &ours, t, |_| true)?;
                    commit::move_ref(repo, MAIN, Some(l), Some(tip), "teahouse sync: replay")?;
                    tip
                };
                specs.insert(0, format!("{MAIN}:{MAIN}"));
                after.push((TRACKING_MAIN.to_string(), Some(tip)));
            }
        }

        if !specs.is_empty() {
            self.upload_attachments()?;
        }
        if let Err(e) = self.push(&specs) {
            // Refused, and the relay's main has moved since our fetch:
            // someone pushed in between. Start over on top of theirs.
            if e.code == Code::RelayRefused
                && specs.iter().any(|s| s.starts_with(MAIN))
                && self.fetch().is_ok()
                && repo.refname_to_id(TRACKING_MAIN).ok() != new_main
            {
                return Err(Again);
            }
            return Err(Stop(e));
        }
        for (name, to) in after {
            set_ref(repo, &name, to)?;
        }
        self.run_pending_purges()?;
        Ok(Outcome::Synced)
    }

    fn endpoint(&self) -> Result<(crate::remote::Endpoint, String)> {
        let remote = self.repo.find_remote("origin")?;
        let url = remote.url().unwrap_or_default().to_string();
        let house = house_of_url(&url).ok_or_else(|| Error::internal(format!("bad URL {url}")))?;
        Ok((crate::remote::Endpoint::of_house_url(&url)?, house))
    }

    /// Every attachment that commits about to be pushed refer to, and that
    /// the relay does not have yet, is uploaded first: the relay refuses a
    /// reference to an attachment it does not hold.
    fn upload_attachments(&self) -> Result<()> {
        let repo = self.repo;
        let mut walk = repo.revwalk()?;
        let mut pushed_any = false;
        for (_, oid) in refs_under(repo, PROPOSALS)? {
            walk.push(oid)?;
            pushed_any = true;
        }
        if let Ok(m) = repo.refname_to_id(MAIN) {
            walk.push(m)?;
            pushed_any = true;
        }
        if !pushed_any {
            return Ok(());
        }
        if let Ok(t) = repo.refname_to_id(TRACKING_MAIN) {
            walk.hide(t)?;
        }
        for (_, oid) in refs_under(repo, TRACKING_PROPOSALS)? {
            walk.hide(oid)?;
        }
        let mut hashes = BTreeSet::new();
        for id in walk {
            let c = repo.find_commit(id?)?;
            let parent = c.parent(0).ok().map(|p| p.tree()).transpose()?;
            let diff = repo.diff_tree_to_tree(parent.as_ref(), Some(&c.tree()?), None)?;
            for d in diff.deltas() {
                if d.status() == Delta::Deleted {
                    continue;
                }
                let path = String::from_utf8_lossy(d.new_file().path_bytes().unwrap_or_default())
                    .to_lowercase();
                if !path.ends_with(".ref.md") {
                    continue;
                }
                let blob = repo.find_blob(d.new_file().id())?;
                if let Some(r) = format::parse_blob_ref(&String::from_utf8_lossy(blob.content())) {
                    if !r.purged {
                        hashes.insert(r.hash);
                    }
                }
            }
        }
        if hashes.is_empty() {
            return Ok(());
        }
        let (endpoint, house) = self.endpoint()?;
        for hash in hashes {
            let has = crate::remote::exec_text(
                &endpoint,
                self.identity,
                self.known_hosts,
                &format!("blob-has {house} {hash}"),
            )?;
            if has.trim() == "yes" {
                continue;
            }
            let path = self.blobs.join(&hash);
            let mut file = std::fs::File::open(&path).map_err(|_| {
                Error::internal(format!(
                    "attachment sha256:{hash} is not in the local cache"
                ))
            })?;
            let size = file.metadata()?.len();
            crate::remote::exec(
                &endpoint,
                self.identity,
                self.known_hosts,
                &format!("blob-put {house} {hash} {size}"),
                &mut file,
                &mut std::io::sink(),
            )?;
        }
        Ok(())
    }

    fn run_pending_purges(&self) -> Result<()> {
        let git_dir = self.repo.path();
        let pending = std::fs::read_to_string(git_dir.join(PENDING_PURGES)).unwrap_or_default();
        if pending.trim().is_empty() {
            return Ok(());
        }
        let (endpoint, house) = self.endpoint()?;
        for hash in pending.lines().filter(|l| !l.is_empty()) {
            crate::remote::exec_text(
                &endpoint,
                self.identity,
                self.known_hosts,
                &format!("blob-purge {house} {hash}"),
            )?;
            remove_pending_purge(git_dir, hash)?;
        }
        Ok(())
    }
}

fn refused(r: &Report) -> Error {
    let mut reasons = r.reasons();
    if reasons.is_empty() {
        reasons = r.refused.iter().map(|(n, s)| format!("{n}: {s}")).collect();
    }
    let mut e = Error::new(
        Code::RelayRefused,
        "the relay refused the change / Relay 拒绝了这次修改",
    );
    e.reasons = reasons;
    e
}

/// The first-parent commits after `base` up to `tip`, oldest first.
pub fn first_parent_since(repo: &Repository, tip: Oid, base: Oid) -> Result<Vec<Oid>> {
    let mut out = Vec::new();
    let mut c = repo.find_commit(tip)?;
    while c.id() != base {
        out.push(c.id());
        match c.parent(0) {
            Ok(p) => c = p,
            Err(_) => break,
        }
    }
    out.reverse();
    Ok(out)
}

/// `problems/0007-x.md`, `works/0007/…`, `talk/0007/…` under another number.
pub fn renumber_path(path: &str, map: &HashMap<String, String>) -> String {
    if let Some((id, _)) = format::parse_problem_path(path) {
        if let Some(n) = map.get(id) {
            return format!("problems/{n}{}", &path["problems/".len() + 4..]);
        }
    }
    for dir in ["works/", "talk/"] {
        if let Some(rest) = path.strip_prefix(dir) {
            if let Some((id, tail)) = rest.split_once('/') {
                if let Some(n) = map.get(id) {
                    return format!("{dir}{n}/{tail}");
                }
            }
        }
    }
    path.to_string()
}

/// Problem numbers in use in a tree.
pub fn problem_ids(repo: &Repository, tree: &git2::Tree<'_>) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    if let Ok(e) = tree.get_path(Path::new("problems")) {
        let sub = repo.find_tree(e.id())?;
        for e in sub.iter() {
            let name = format!("problems/{}", String::from_utf8_lossy(e.name_bytes()));
            if let Some((id, _)) = format::parse_problem_path(&name) {
                ids.insert(id.to_string());
            }
        }
    }
    Ok(ids)
}

/// The problem file a number names in a tree, if any.
fn problem_file(repo: &Repository, tree: &git2::Tree<'_>, id: &str) -> Result<Option<String>> {
    if let Ok(e) = tree.get_path(Path::new("problems")) {
        let sub = repo.find_tree(e.id())?;
        for e in sub.iter() {
            let name = format!("problems/{}", String::from_utf8_lossy(e.name_bytes()));
            if format::parse_problem_path(&name).is_some_and(|(i, _)| i == id) {
                return Ok(Some(name));
            }
        }
    }
    Ok(None)
}

pub fn next_id(ids: &BTreeSet<String>) -> String {
    let max = ids
        .iter()
        .filter_map(|i| i.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    format!("{:04}", max + 1)
}

/// Rewrites `Problem: <old>` trailers through `map`, and adds
/// `Renumbered:` lines for `new_here`.
pub fn renumber_message(
    msg: &str,
    map: &HashMap<String, String>,
    new_here: &[(String, String)],
) -> String {
    let mut out: Vec<String> = msg
        .lines()
        .map(|l| match l.strip_prefix("Problem: ") {
            Some(v) => match map.get(v.trim()) {
                Some(n) => format!("Problem: {n}"),
                None => l.to_string(),
            },
            None => l.to_string(),
        })
        .collect();
    while out.last().is_some_and(|l| l.is_empty()) {
        out.pop();
    }
    for (a, b) in new_here {
        out.push(format!("Renumbered: {a} -> {b}"));
    }
    out.join("\n") + "\n"
}

/// Applies each commit of `commits` (oldest first) on top of `onto`, as a
/// new signed commit, keeping second parents of merges. Paths whose blob on
/// the new base differs from the one the commit was made on are conflicts.
/// A commit adding a problem whose number is taken on the new base moves to
/// the next free number (with its `works/` and `talk/`), and so do later
/// commits. `keep` filters which commits are replayed. Returns the new tip
/// and old → new ids.
pub fn replay(
    repo: &Repository,
    identity: &Identity,
    commits: &[Oid],
    onto: Oid,
    keep: impl Fn(Oid) -> bool,
) -> Result<(Oid, Vec<(Oid, Oid)>)> {
    let mut tip = repo.find_commit(onto)?;
    let mut map: HashMap<String, String> = HashMap::new();
    let mut moved = Vec::new();
    for &id in commits {
        if !keep(id) {
            continue;
        }
        let c = repo.find_commit(id)?;
        let parent_tree = match c.parent(0) {
            Ok(p) => Some(p.tree()?),
            Err(_) => None,
        };
        let tree = c.tree()?;
        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
        let base = tip.tree()?;
        let mut ids = problem_ids(repo, &base)?;
        let mut new_here = Vec::new();
        // New problems first: their number may have to move.
        for d in diff.deltas() {
            if d.status() != Delta::Added {
                continue;
            }
            let path =
                String::from_utf8_lossy(d.new_file().path_bytes().unwrap_or_default()).into_owned();
            let path = renumber_path(&path, &map);
            if let Some((pid, _)) = format::parse_problem_path(&path) {
                if problem_file(repo, &base, pid)?.is_some_and(|f| f != path) {
                    let n = next_id(&ids);
                    ids.insert(n.clone());
                    let original = format::parse_problem_path(&String::from_utf8_lossy(
                        d.new_file().path_bytes().unwrap_or_default(),
                    ))
                    .map(|(i, _)| i.to_string())
                    .unwrap_or_default();
                    map.insert(original.clone(), n.clone());
                    new_here.push((original, n));
                }
            }
        }
        let mut b = TreeUpdateBuilder::new();
        for d in diff.deltas() {
            let (old_f, new_f) = (d.old_file(), d.new_file());
            let src = if d.status() == Delta::Deleted {
                &old_f
            } else {
                &new_f
            };
            let path = renumber_path(
                &String::from_utf8_lossy(src.path_bytes().unwrap_or_default()),
                &map,
            );
            let on_base = base.get_path(Path::new(&path)).ok().map(|e| e.id());
            let before = (d.status() != Delta::Added).then(|| old_f.id());
            let after = (d.status() != Delta::Deleted).then(|| new_f.id());
            if on_base != before && on_base != after {
                return Err(Error::new(
                    Code::Conflict,
                    format!("{path} was changed by someone else meanwhile / {path} 已被别人改动"),
                )
                .with("path", &path));
            }
            match after {
                Some(oid) => {
                    b.upsert(path.as_str(), oid, new_f.mode());
                }
                None => {
                    b.remove(path.as_str());
                }
            }
        }
        let new_tree = b.create_updated(repo, &base)?;
        let msg = renumber_message(&String::from_utf8_lossy(c.message_bytes()), &map, &new_here);
        let mut parents = vec![tip.clone()];
        for p in c.parents().skip(1) {
            parents.push(p);
        }
        let parent_refs: Vec<&git2::Commit<'_>> = parents.iter().collect();
        let new_id =
            commit::signed_commit(repo, identity, &c.author(), new_tree, &parent_refs, &msg)?;
        moved.push((id, new_id));
        tip = repo.find_commit(new_id)?;
    }
    Ok((tip.id(), moved))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renumbering() {
        let map: HashMap<String, String> = [("0007".to_string(), "0009".to_string())].into();
        assert_eq!(
            renumber_path("problems/0007-停车.md", &map),
            "problems/0009-停车.md"
        );
        assert_eq!(renumber_path("problems/0007.md", &map), "problems/0009.md");
        assert_eq!(
            renumber_path("works/0007/a/b.md", &map),
            "works/0009/a/b.md"
        );
        assert_eq!(
            renumber_path("talk/0007/20261018T000000Z-li.md", &map),
            "talk/0009/20261018T000000Z-li.md"
        );
        assert_eq!(
            renumber_path("talk/board/n/x.md", &map),
            "talk/board/n/x.md"
        );
        assert_eq!(
            renumber_path("problems/0008-x.md", &map),
            "problems/0008-x.md"
        );
        assert_eq!(
            renumber_message(
                "新问题\n\nProblem: 0007\nAct: open-problem\n",
                &map,
                &[("0007".into(), "0009".into())]
            ),
            "新问题\n\nProblem: 0009\nAct: open-problem\nRenumbered: 0007 -> 0009\n"
        );
        let ids: BTreeSet<String> = ["0001".into(), "0012".into()].into();
        assert_eq!(next_id(&ids), "0013");
        assert_eq!(next_id(&BTreeSet::new()), "0001");
    }
}
