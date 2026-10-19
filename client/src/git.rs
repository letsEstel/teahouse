//! The engine's view of a local clone, through libgit2 (`git2`).
//!
//! The engine never runs an external git program: it must compile into the
//! Android app unchanged (CLIENT-API.md §2, §7). Everything here reads
//! committed objects, never the working tree, and answers the same questions
//! the relay asks `git` itself (first-parent steps, trailers, diffs), so both
//! sides agree on what a commit is. `tests/read.rs` builds its clones with
//! the git program to keep the two in step. Rename detection stays off
//! everywhere: a move is a deletion plus an addition, as the relay's
//! `--no-renames` sees it.

use crate::error::{Error, Result};
use git2::{Commit, Delta, DiffOptions, ObjectType, Oid, Patch, Repository, Sort, Tree};
use std::path::Path;

/// One file of a tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeFile {
    pub path: String,
    /// `100644`, `100755`, `120000` (symlink), `160000` (submodule).
    pub mode: String,
    pub oid: String,
    /// `None` for submodules.
    pub size: Option<u64>,
}

/// What a single commit says about itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitInfo {
    pub subject: String,
    pub body: String,
    pub time: i64,
    pub problem: Vec<String>,
}

/// One commit on the first-parent chain of `main`, with what it changed
/// compared with its first parent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub hash: String,
    pub parents: Vec<String>,
    pub author_name: String,
    pub author_email: String,
    /// Committer time, seconds since the epoch.
    pub time: i64,
    pub subject: String,
    pub problem: Vec<String>,
    pub act: Vec<String>,
    pub status: Vec<String>,
    pub proposal: Vec<String>,
    /// `(A|M|D|T, path)`, compared with the first parent.
    pub changes: Vec<(char, String)>,
}

impl From<git2::Error> for Error {
    fn from(e: git2::Error) -> Error {
        Error::internal(format!("git: {}", e.message()))
    }
}

pub struct Git {
    repo: Repository,
}

fn lossy(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

fn oid(s: &str) -> Result<Oid> {
    Oid::from_str(s).map_err(|_| Error::internal(format!("not an object id: {s}")))
}

fn not_found(e: &git2::Error) -> bool {
    e.code() == git2::ErrorCode::NotFound
}

/// Values of `key:` trailers, matched without regard to case as git does,
/// with folded lines joined.
fn trailer_values(message: &[u8], key: &str) -> Vec<String> {
    let Ok(t) = git2::message_trailers_bytes(message) else {
        return Vec::new();
    };
    t.iter()
        .filter(|(k, _)| lossy(k).trim().eq_ignore_ascii_case(key))
        .map(|(_, v)| lossy(v).split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|v| !v.is_empty())
        .collect()
}

/// The subject (first paragraph, unfolded) and the body after it, as git's
/// `%s` and `%b` give them.
fn split_message(message: &[u8]) -> (String, String) {
    let m = lossy(message);
    let m = m.trim_start_matches('\n');
    let (subject, body) = match m.find("\n\n") {
        Some(i) => (&m[..i], &m[i + 2..]),
        None => (m, ""),
    };
    let subject = subject.lines().map(str::trim).collect::<Vec<_>>().join(" ");
    (subject.trim().to_string(), body.trim_end().to_string())
}

fn status_char(d: Delta) -> char {
    match d {
        Delta::Added => 'A',
        Delta::Deleted => 'D',
        Delta::Typechange => 'T',
        _ => 'M',
    }
}

impl Git {
    pub fn open(dir: &Path) -> Result<Git> {
        Ok(Git {
            repo: Repository::open(dir)?,
        })
    }

    pub fn repo(&self) -> &Repository {
        &self.repo
    }

    /// The git directory (`.git` of a clone, or the bare repository).
    pub fn dir(&self) -> &Path {
        self.repo.path()
    }

    fn commit(&self, rev: &str) -> Result<Commit<'_>> {
        Ok(self.repo.find_commit(oid(rev)?)?)
    }

    fn tree_of(&self, rev: Option<&str>) -> Result<Option<Tree<'_>>> {
        rev.map(|r| Ok(self.commit(r)?.tree()?)).transpose()
    }

    /// The commit a full ref name names, or `None`.
    pub fn resolve(&self, refname: &str) -> Result<Option<String>> {
        match self.repo.find_reference(refname) {
            Ok(r) => Ok(r.peel_to_commit().ok().map(|c| c.id().to_string())),
            Err(e) if not_found(&e) || e.code() == git2::ErrorCode::InvalidSpec => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Every file of `rev`'s tree.
    pub fn ls_tree(&self, rev: &str) -> Result<Vec<TreeFile>> {
        let tree = self.commit(rev)?.tree()?;
        let odb = self.repo.odb()?;
        let mut files = Vec::new();
        let mut failed = None;
        tree.walk(git2::TreeWalkMode::PreOrder, |dir, e| {
            let kind = e.kind();
            if kind == Some(ObjectType::Tree) {
                return git2::TreeWalkResult::Ok;
            }
            let size = match kind {
                Some(ObjectType::Blob) => match odb.read_header(e.id()) {
                    Ok((n, _)) => Some(n as u64),
                    Err(err) => {
                        failed = Some(err);
                        return git2::TreeWalkResult::Abort;
                    }
                },
                _ => None,
            };
            files.push(TreeFile {
                path: format!("{dir}{}", lossy(e.name_bytes())),
                mode: format!("{:06o}", e.filemode()),
                oid: e.id().to_string(),
                size,
            });
            git2::TreeWalkResult::Ok
        })?;
        if let Some(e) = failed {
            return Err(e.into());
        }
        Ok(files)
    }

    /// The contents of blobs by object id, in order; `None` where an object
    /// is missing or not a blob.
    pub fn read_blobs(&self, oids: &[String]) -> Result<Vec<Option<Vec<u8>>>> {
        oids.iter()
            .map(|o| match self.repo.find_blob(oid(o)?) {
                Ok(b) => Ok(Some(b.content().to_vec())),
                Err(e) if not_found(&e) || e.code() == git2::ErrorCode::Invalid => Ok(None),
                Err(e) => Err(e.into()),
            })
            .collect()
    }

    /// The content of the file at `path` in `rev`, or `None`.
    pub fn read_path(&self, rev: &str, path: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.blob_at(Some(rev), path)?.map(|b| b.content().to_vec()))
    }

    fn blob_at(&self, rev: Option<&str>, path: &str) -> Result<Option<git2::Blob<'_>>> {
        let Some(tree) = self.tree_of(rev)? else {
            return Ok(None);
        };
        match tree.get_path(Path::new(path)) {
            Ok(e) if e.kind() == Some(ObjectType::Blob) => Ok(Some(self.repo.find_blob(e.id())?)),
            Ok(_) => Ok(None),
            Err(e) if not_found(&e) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Subject, body, committer time and `Problem:` trailers of one commit.
    pub fn commit_info(&self, rev: &str) -> Result<CommitInfo> {
        let c = self.commit(rev)?;
        let (subject, body) = split_message(c.message_bytes());
        let time = c.committer().when().seconds();
        Ok(CommitInfo {
            subject,
            body,
            time,
            problem: trailer_values(c.message_bytes(), "Problem"),
        })
    }

    /// `(A|M|D|T, path)` from tree `a` (`None`: nothing) to tree `b`.
    fn changes(&self, a: Option<&Tree<'_>>, b: &Tree<'_>) -> Result<Vec<(char, String)>> {
        let mut opts = DiffOptions::new();
        let diff = self.repo.diff_tree_to_tree(a, Some(b), Some(&mut opts))?;
        Ok(diff
            .deltas()
            .map(|d| {
                let f = if d.status() == Delta::Deleted {
                    d.old_file()
                } else {
                    d.new_file()
                };
                (
                    status_char(d.status()),
                    lossy(f.path_bytes().unwrap_or_default()),
                )
            })
            .collect())
    }

    /// The first-parent chain of `rev`, oldest first, each step with the
    /// paths it changed compared with its first parent (so a merge shows what
    /// it brought in).
    pub fn first_parent_steps(&self, rev: &str) -> Result<Vec<Step>> {
        let mut walk = self.repo.revwalk()?;
        walk.push(oid(rev)?)?;
        walk.simplify_first_parent()?;
        let mut ids = walk.collect::<std::result::Result<Vec<Oid>, _>>()?;
        ids.reverse();
        let mut steps = Vec::with_capacity(ids.len());
        let mut prev_tree: Option<Tree<'_>> = None;
        for id in ids {
            let c = self.repo.find_commit(id)?;
            let tree = c.tree()?;
            // The previous step is this one's first parent, so its tree is
            // already at hand.
            let parent_tree = match (c.parent_ids().next(), prev_tree.take()) {
                (None, _) => None,
                (Some(_), Some(t)) => Some(t),
                (Some(p), None) => Some(self.repo.find_commit(p)?.tree()?),
            };
            let msg = c.message_bytes();
            let (subject, _) = split_message(msg);
            let author = c.author();
            steps.push(Step {
                hash: id.to_string(),
                parents: c.parent_ids().map(|p| p.to_string()).collect(),
                author_name: lossy(author.name_bytes()),
                author_email: lossy(author.email_bytes()),
                time: c.committer().when().seconds(),
                subject,
                problem: trailer_values(msg, "Problem"),
                act: trailer_values(msg, "Act"),
                status: trailer_values(msg, "Status"),
                proposal: trailer_values(msg, "Proposal"),
                changes: self.changes(parent_tree.as_ref(), &tree)?,
            });
            prev_tree = Some(tree);
        }
        Ok(steps)
    }

    fn walk(&self, exclude: &str, include: &str) -> Result<git2::Revwalk<'_>> {
        let mut walk = self.repo.revwalk()?;
        walk.push(oid(include)?)?;
        walk.hide(oid(exclude)?)?;
        Ok(walk)
    }

    /// How many commits `exclude..include` has.
    pub fn count(&self, exclude: &str, include: &str) -> Result<usize> {
        let mut n = 0;
        for r in self.walk(exclude, include)? {
            r?;
            n += 1;
        }
        Ok(n)
    }

    /// The commits of `exclude..include`, parents before children.
    pub fn rev_list_reverse(&self, exclude: &str, include: &str) -> Result<Vec<String>> {
        let mut walk = self.walk(exclude, include)?;
        walk.set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)?;
        walk.map(|r| Ok(r?.to_string())).collect()
    }

    /// `(refname, commit)` for every ref under `prefix`, sorted by name.
    pub fn refs(&self, prefix: &str) -> Result<Vec<(String, String)>> {
        let mut out = Vec::new();
        for r in self.repo.references()? {
            let r = r?;
            let Some(name) = r.name().ok().filter(|n| n.starts_with(prefix)) else {
                continue;
            };
            if let Ok(c) = r.peel_to_commit() {
                out.push((name.to_string(), c.id().to_string()));
            }
        }
        out.sort();
        Ok(out)
    }

    pub fn merge_base(&self, a: &str, b: &str) -> Result<Option<String>> {
        match self.repo.merge_base(oid(a)?, oid(b)?) {
            Ok(o) => Ok(Some(o.to_string())),
            Err(e) if not_found(&e) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// `(A|M|D|T, path)` between two commits (`a`: `None` for nothing).
    pub fn diff_paths(&self, a: Option<&str>, b: &str) -> Result<Vec<(char, String)>> {
        let ta = self.tree_of(a)?;
        let tb = self.commit(b)?.tree()?;
        self.changes(ta.as_ref(), &tb)
    }

    /// The line diff of one path between two commits (`a`: `None` for
    /// nothing), with all context: `(' '|'+'|'-', text)`. `None` for binary
    /// files.
    pub fn line_diff(
        &self,
        a: Option<&str>,
        b: &str,
        path: &str,
    ) -> Result<Option<Vec<(char, String)>>> {
        let (old, new) = (self.blob_at(a, path)?, self.blob_at(Some(b), path)?);
        if [&old, &new]
            .iter()
            .any(|x| x.as_ref().is_some_and(|b| b.is_binary()))
        {
            return Ok(None);
        }
        let mut opts = DiffOptions::new();
        opts.context_lines(u32::MAX / 2);
        let p = Path::new(path);
        let old_bytes = old.as_ref().map_or(&[][..], |b| b.content());
        let new_bytes = new.as_ref().map_or(&[][..], |b| b.content());
        let patch = Patch::from_buffers(old_bytes, Some(p), new_bytes, Some(p), Some(&mut opts))?;
        let mut lines = Vec::new();
        for h in 0..patch.num_hunks() {
            for l in 0..patch.num_lines_in_hunk(h)? {
                let line = patch.line_in_hunk(h, l)?;
                let c = line.origin();
                if matches!(c, ' ' | '+' | '-') {
                    let text = lossy(line.content());
                    lines.push((c, text.strip_suffix('\n').unwrap_or(&text).to_string()));
                }
            }
        }
        Ok(Some(lines))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages() {
        let m = b"Subject line\n\nBody one.\n\nProblem: 0001\nact: note\nProposal: zhao/x\n";
        assert_eq!(
            split_message(m),
            (
                "Subject line".into(),
                "Body one.\n\nProblem: 0001\nact: note\nProposal: zhao/x".into()
            )
        );
        assert_eq!(trailer_values(m, "Problem"), ["0001"]);
        assert_eq!(trailer_values(m, "Act"), ["note"]);
        assert!(trailer_values(b"Only a subject\n", "Problem").is_empty());
        assert!(trailer_values(
            b"Subject\n\nProblem: 0001 is a body line\nhere\n",
            "Problem"
        )
        .is_empty());
        assert_eq!(split_message(b"two\nlines\n").0, "two lines");
    }
}
