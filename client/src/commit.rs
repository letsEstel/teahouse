//! Making commits the way every client must: on top of a known tree, with
//! the `Problem:` and `Act:` trailers of CLIENT-API.md §1.4, signed with the
//! device key in git's ssh signature format.

use crate::error::{Error, Result};
use crate::identity::Identity;
use git2::{build::TreeUpdateBuilder, Commit, FileMode, Oid, Repository, Signature, Time, Tree};

/// One change to a tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    Put(String, Vec<u8>),
    Remove(String),
}

impl Edit {
    pub fn path(&self) -> &str {
        match self {
            Edit::Put(p, _) | Edit::Remove(p) => p,
        }
    }
}

/// Who a commit is by: `<name> <<member-id>@teahouse>` (§1.4, so that a
/// commit names its member even without the seating chart at hand).
pub struct Author {
    pub id: String,
    pub name: String,
}

impl Author {
    pub fn signature(&self, at: i64) -> Result<Signature<'static>> {
        Ok(Signature::new(
            &self.name,
            &format!("{}@teahouse", self.id),
            &Time::new(at, 0),
        )?)
    }
}

/// `base` with `edits` applied (`base`: `None` for an empty tree).
pub fn tree_with(repo: &Repository, base: Option<&Tree<'_>>, edits: &[Edit]) -> Result<Oid> {
    let empty;
    let base = match base {
        Some(t) => t,
        None => {
            empty = repo.find_tree(repo.treebuilder(None)?.write()?)?;
            &empty
        }
    };
    let mut b = TreeUpdateBuilder::new();
    for e in edits {
        match e {
            Edit::Put(p, data) => {
                let oid = repo.blob(data)?;
                b.upsert(p.as_str(), oid, FileMode::Blob);
            }
            Edit::Remove(p) => {
                b.remove(p.as_str());
            }
        }
    }
    Ok(b.create_updated(repo, base)?)
}

/// A commit message: subject, optional body, then the trailer block.
pub fn message(subject: &str, body: Option<&str>, trailers: &[(&str, String)]) -> String {
    let subject = subject.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut m = if subject.is_empty() {
        "…".to_string()
    } else {
        subject
    };
    m.push_str("\n\n");
    if let Some(b) = body.map(str::trim).filter(|b| !b.is_empty()) {
        m.push_str(b);
        m.push_str("\n\n");
    }
    for (k, v) in trailers {
        m.push_str(&format!("{k}: {v}\n"));
    }
    m
}

/// Writes a commit signed with the device key and returns its id. No ref
/// is moved.
pub fn signed_commit(
    repo: &Repository,
    identity: &Identity,
    author: &Signature<'_>,
    tree: Oid,
    parents: &[&Commit<'_>],
    message: &str,
) -> Result<Oid> {
    let tree = repo.find_tree(tree)?;
    let buf = repo.commit_create_buffer(author, author, message, &tree, parents)?;
    let content =
        std::str::from_utf8(&buf).map_err(|_| Error::internal("commit buffer is not UTF-8"))?;
    let sig = identity.sign(content.as_bytes())?;
    Ok(repo.commit_signed(content, &sig, Some("gpgsig"))?)
}

/// Moves `refname` from `old` (`None`: must not exist) to `new` (`None`:
/// delete), refusing if someone moved it meanwhile.
pub fn move_ref(
    repo: &Repository,
    refname: &str,
    old: Option<Oid>,
    new: Option<Oid>,
    why: &str,
) -> Result<()> {
    let current = repo.refname_to_id(refname).ok();
    if current != old {
        return Err(Error::new(
            crate::error::Code::Conflict,
            format!("{refname} changed while writing / 写入时 {refname} 被改动了"),
        ));
    }
    match (new, current) {
        (Some(n), Some(o)) => {
            repo.reference_matching(refname, n, true, o, why)?;
        }
        (Some(n), None) => {
            repo.reference(refname, n, false, why)?;
        }
        (None, Some(_)) => {
            repo.find_reference(refname)?.delete()?;
        }
        (None, None) => {}
    }
    Ok(())
}
