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
        let result = transport::with_key(self.identity, &report, || {
            let mut opts = FetchOptions::new();
            opts.remote_callbacks(transport::callbacks(
                self.identity,
                self.known_hosts,
                report.clone(),
            ));
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
        let result = transport::with_key(self.identity, &report, || {
            let mut opts = PushOptions::new();
            opts.remote_callbacks(transport::callbacks(
                self.identity,
                self.known_hosts,
                report.clone(),
            ));
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
        let result = self.sync_inner();
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
                record.message = Some(e.message.clone());
            }
        }
        record.save(&git_dir);
        result
    }

    fn sync_inner(&self) -> Result<Outcome> {
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
                return Err(Error::internal("the relay has no main for this teahouse"));
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

        self.push(&specs)?;
        for (name, to) in after {
            set_ref(repo, &name, to)?;
        }
        Ok(Outcome::Synced)
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
