//! `export` (CLIENT-API.md §4): the whole teahouse, for keeping or for
//! taking elsewhere, in a folder:
//!
//! ```text
//! <to>/<house>-<YYYYMMDD>/
//!   <house>.bundle    every commit of main and of open proposals (git bundle v2)
//!   blobs/<sha256>    every attachment this device has
//!   files/…           main as plain files, attachments next to their references
//!   README.txt        what all this is
//! ```
//!
//! A teahouse can be closed but not killed: anyone with this folder can
//! `git clone <house>.bundle` and open it again at another relay.

use crate::error::{Error, Result};
use crate::format;
use crate::house::Snapshot;
use git2::{ObjectType, Oid, TreeWalkMode, TreeWalkResult};
use std::fs;
use std::path::{Path, PathBuf};

const README: &str = "这是一间茶馆的完整导出。This is a complete export of a teahouse.

<house>.bundle  完整的 git 历史（main 和未处理的提案）。用 `git clone <house>.bundle` 打开。
                The full git history (main and open proposals). Open it with `git clone`.
blobs/          附件，按内容的 SHA-256 命名。Attachments, named by the SHA-256 of their content.
files/          main 的内容，可以直接阅读；附件放在它的引用文件旁边。
                The contents of main as plain files; attachments next to their references.
";

/// Writes a git bundle (v2) of `refs` to `path`: the header names each ref,
/// the pack holds every object they reach.
pub fn write_bundle(repo: &git2::Repository, refs: &[(String, Oid)], path: &Path) -> Result<()> {
    let mut pb = repo.packbuilder()?;
    let mut walk = repo.revwalk()?;
    for (_, oid) in refs {
        walk.push(*oid)?;
    }
    pb.insert_walk(&mut walk)?;
    let mut pack = git2::Buf::new();
    pb.write_buf(&mut pack)?;
    let mut out = b"# v2 git bundle\n".to_vec();
    for (name, oid) in refs {
        out.extend_from_slice(format!("{oid} {name}\n").as_bytes());
    }
    // So that `git clone` checks out main, as `git bundle create --all` does.
    if let Some((_, main)) = refs.iter().find(|(n, _)| n == "refs/heads/main") {
        out.extend_from_slice(format!("{main} HEAD\n").as_bytes());
    }
    out.push(b'\n');
    out.extend_from_slice(&pack);
    fs::write(path, out)?;
    Ok(())
}

/// A folder name that does not exist yet under `to`.
fn free_dir(to: &Path, stem: &str) -> PathBuf {
    let mut n = 1;
    loop {
        let p = if n == 1 {
            to.join(stem)
        } else {
            to.join(format!("{stem}-{n}"))
        };
        if !p.exists() {
            return p;
        }
        n += 1;
    }
}

pub fn export(snap: &Snapshot, blobs: &Path, to: &Path, today: &str) -> Result<PathBuf> {
    let repo = snap.git.repo();
    fs::create_dir_all(to).map_err(|e| {
        Error::invalid(format!("{}: {e}", to.display())).with("to", to.display().to_string())
    })?;
    let dir = free_dir(to, &format!("{}-{today}", snap.id));
    fs::create_dir_all(dir.join("files"))?;
    fs::create_dir_all(dir.join("blobs"))?;
    fs::write(dir.join("README.txt"), README)?;

    let mut refs = vec![("refs/heads/main".to_string(), Oid::from_str(snap.head())?)];
    for r in repo.references_glob("refs/proposals/*")? {
        let r = r?;
        if let (Ok(name), Some(oid)) = (r.name(), r.target()) {
            refs.push((name.to_string(), oid));
        }
    }
    write_bundle(repo, &refs, &dir.join(format!("{}.bundle", snap.id)))?;

    // main as plain files, and the attachments this device has.
    let tree = repo.find_commit(Oid::from_str(snap.head())?)?.tree()?;
    let mut skipped = Vec::new();
    let mut failure = None;
    tree.walk(TreeWalkMode::PreOrder, |parent, e| {
        if e.kind() != Some(ObjectType::Blob) || e.filemode() == 0o120000 {
            return TreeWalkResult::Ok;
        }
        let rel = format!("{parent}{}", String::from_utf8_lossy(e.name_bytes()));
        let write = || -> Result<()> {
            let data = repo.find_blob(e.id())?;
            let dest = dir.join("files").join(&rel);
            if let Some(p) = dest.parent() {
                fs::create_dir_all(p)?;
            }
            fs::write(&dest, data.content())?;
            if rel.to_lowercase().ends_with(".ref.md") {
                if let Some(r) = format::parse_blob_ref(&String::from_utf8_lossy(data.content())) {
                    let cached = blobs.join(&r.hash);
                    if !r.purged && cached.is_file() {
                        fs::copy(&cached, dir.join("blobs").join(&r.hash))?;
                        fs::copy(&cached, dest.with_file_name(format::file_title(&rel)))?;
                    }
                }
            }
            Ok(())
        };
        match write() {
            Ok(()) => {}
            // A name this file system cannot hold (e.g. `:` on Windows):
            // the bundle still has it.
            Err(e)
                if e.code == crate::error::Code::Internal
                    && rel.contains(['\\', ':', '*', '?', '"', '<', '>', '|']) =>
            {
                skipped.push(rel);
            }
            Err(e) => {
                failure = Some(e);
                return TreeWalkResult::Abort;
            }
        }
        TreeWalkResult::Ok
    })?;
    if let Some(e) = failure {
        return Err(e);
    }
    if !skipped.is_empty() {
        fs::write(dir.join("files-skipped.txt"), skipped.join("\n") + "\n")?;
    }
    Ok(dir)
}
