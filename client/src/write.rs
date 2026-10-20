//! The write commands of CLIENT-API.md §4: write locally first, then try to
//! push. Every commit is signed with the device key.
//!
//! * Preflight: the engine checks roles and texts with `teahouse-core`
//!   before writing, so most refusals never reach the relay (`forbidden`,
//!   `invalid`, `too_large`; nothing is written).
//! * A guest's content command (`openProblem`, `addWork`, `updateWork`,
//!   `postNotice`) becomes a proposal (`becameProposal: true`); management
//!   commands never do.
//! * Offline: the commit stays local (`commit: "local"`) and goes out with
//!   the next sync.
//! * Refused by the relay: the commit is taken back out of the local
//!   history and what the person wrote is kept in `<root>/drafts/`.

use crate::commit::{self, Author, Edit};
use crate::error::{Code, Error, Result};
use crate::format;
use crate::house::Snapshot;
use crate::identity::Identity;
use crate::model::{Status, SyncState, WorkKind};
use crate::sync::{self, Link, MAIN, PROPOSALS};
use crate::Client;
use git2::{Oid, Repository};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use teahouse_core::rules::{self, check_text, format_stamp};
use teahouse_core::{Member, Members, Role, MEMBERS_PATH};

/// Largest attachment (CLIENT-API.md §1.6).
pub const MAX_BLOB: u64 = 100 * 1024 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteResult {
    /// `local` or `pushed`.
    pub commit: &'static str,
    pub sync: SyncState,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub became_proposal: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SectionArg {
    #[serde(default)]
    pub heading: String,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeArg {
    pub id: Option<String>,
    pub kind: Option<WorkKindArg>,
    pub title: Option<String>,
    pub text: Option<String>,
    pub from_path: Option<String>,
    #[serde(default)]
    pub delete: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkKindArg {
    Doc,
    Plan,
    Table,
    Image,
    File,
}

/// What to undo if the relay refuses.
enum Undo {
    /// The commit made on `main` (on top of the previous `main`).
    Main(Oid),
    /// A proposal ref and what it pointed at before.
    Proposal(String, Option<Oid>),
    /// An attachment queued for deletion at the relay.
    PendingPurge(String),
}

pub struct Writer<'c> {
    client: &'c Client,
    house: String,
    pub snap: Snapshot,
    identity: Identity,
    me: Member,
}

fn forbidden(what: &str) -> Error {
    Error::new(
        Code::Forbidden,
        format!("your seat does not allow this: {what} / 你的座位不能做这件事"),
    )
    .with("action", what)
}

fn invalid(msg: impl Into<String>) -> Error {
    Error::invalid(msg)
}

/// A file name from a title: letters and digits of any script, `-` and `_`;
/// spaces become `-`. Never empty.
pub fn slug(title: &str, max: usize) -> String {
    let mut s = String::new();
    for c in title.trim().chars() {
        if c.is_alphanumeric() || c == '_' {
            s.push(c);
        } else if (c.is_whitespace() || c == '-' || c == '.') && !s.ends_with('-') {
            s.push('-');
        }
    }
    let s: String = s.trim_matches('-').chars().take(max).collect();
    let s = s.trim_end_matches('-').to_string();
    if s.is_empty() {
        "untitled".into()
    } else {
        s
    }
}

/// An ASCII name (proposal names, notice names): `[a-z0-9-]`.
pub fn ascii_slug(title: &str, max: usize) -> String {
    let mut s = String::new();
    for c in title.trim().chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase());
        } else if !s.ends_with('-') && !s.is_empty() {
            s.push('-');
        }
    }
    let s: String = s.trim_matches('-').chars().take(max).collect();
    s.trim_end_matches('-').to_string()
}

/// Four random characters for names that must not collide.
pub(crate) fn suffix() -> String {
    use ssh_key::rand_core::RngCore;
    const ALPHABET: &[u8] = b"abcdefghjkmnpqrstvwxyz23456789";
    let mut b = [0u8; 4];
    ssh_key::rand_core::OsRng.fill_bytes(&mut b);
    b.iter()
        .map(|x| ALPHABET[*x as usize % ALPHABET.len()] as char)
        .collect()
}

fn mime_of(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "heic" => "image/heic",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "mp3" => "audio/mpeg",
        "m4a" => "audio/mp4",
        "mp4" => "video/mp4",
        "txt" => "text/plain",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => "application/octet-stream",
    }
}

fn render_problem(title: &str, status: &str, body: &[SectionArg]) -> String {
    let mut s = format::render_frontmatter(&[("title", title), ("status", status)]);
    for sec in body {
        let text = sec.text.trim_matches('\n');
        if sec.heading.trim().is_empty() {
            if !text.is_empty() {
                s.push_str(text);
                s.push_str("\n\n");
            }
        } else {
            s.push_str(&format!("## {}\n", sec.heading.trim()));
            if !text.is_empty() {
                s.push_str(text);
                s.push('\n');
            }
            s.push('\n');
        }
    }
    s.trim_end_matches('\n').to_string() + "\n"
}

/// The first line of a text, at most `n` characters, for a commit subject.
fn first_line(text: &str, n: usize) -> String {
    let l = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    let mut s: String = l.chars().take(n).collect();
    if l.chars().count() > n {
        s.push('…');
    }
    s
}

impl<'c> Writer<'c> {
    pub fn open(client: &'c Client, house: &str) -> Result<Writer<'c>> {
        let identity = client.identity()?;
        let snap = client.snapshot(house)?;
        let me = snap.me.clone().ok_or_else(|| {
            Error::new(
                Code::NotAMember,
                "this device has no seat in this teahouse / 这台设备在这间茶馆里没有座位",
            )
            .with("house", house)
        })?;
        Ok(Writer {
            client,
            house: house.to_string(),
            snap,
            identity,
            me,
        })
    }

    fn repo(&self) -> &Repository {
        self.snap.git.repo()
    }

    fn author(&self) -> Result<git2::Signature<'static>> {
        Author {
            id: self.me.id.clone(),
            name: self.me.name.clone(),
        }
        .signature(sync::now())
    }

    fn role(&self) -> Role {
        self.me.role
    }

    fn require(&self, roles: &[Role], what: &str) -> Result<()> {
        if roles.contains(&self.role()) {
            Ok(())
        } else {
            Err(forbidden(what))
        }
    }

    fn head(&self) -> Result<git2::Commit<'_>> {
        Ok(self.repo().find_commit(Oid::from_str(self.snap.head())?)?)
    }

    fn problem_exists(&self, pid: &str) -> bool {
        self.snap.problem_path(pid).is_some()
    }

    fn require_problem(&self, pid: &str) -> Result<()> {
        if self.problem_exists(pid) {
            Ok(())
        } else {
            Err(Error::not_found(format!("no problem {pid} / 没有问题 {pid}")).with("problem", pid))
        }
    }

    /// A path under `dir` not yet on `main`: `<stem><ext>`, `<stem>-2<ext>`, …
    fn free_path(&self, dir: &str, stem: &str, ext: &str) -> String {
        let mut n = 1;
        loop {
            let p = if n == 1 {
                format!("{dir}{stem}{ext}")
            } else {
                format!("{dir}{stem}-{n}{ext}")
            };
            if self.snap.entry(&p).is_none()
                && !(ext == ".csv" && self.snap.entry(&format!("{p}.meta.md")).is_some())
            {
                return p;
            }
            n += 1;
        }
    }

    fn next_problem_id(&self) -> String {
        let ids = self
            .snap
            .problem_files()
            .into_iter()
            .map(|(id, _, _)| id.to_string())
            .collect();
        sync::next_id(&ids)
    }

    // -- making commits ---------------------------------------------------

    /// Commits `edits` on top of `main` and moves `main`.
    fn commit_main(
        &self,
        edits: &[Edit],
        subject: &str,
        body: Option<&str>,
        trailers: &[(&str, String)],
    ) -> Result<Oid> {
        // The steward and host write main directly; still, everything a
        // commit touches must be theirs to touch (law 6).
        for e in edits {
            let p = e.path();
            // Guests add their own posts (law 4); that is all.
            let own_post = matches!(e, Edit::Put(..))
                && rules::parse_talk_path(p).is_some_and(|t| t.author == self.me.id);
            if !own_post && !rules::may_write_on_main(self.role(), p) {
                return Err(forbidden(p));
            }
        }
        let head = self.head()?;
        let tree = commit::tree_with(self.repo(), Some(&head.tree()?), edits)?;
        let msg = commit::message(subject, body, trailers);
        let c = commit::signed_commit(
            self.repo(),
            &self.identity,
            &self.author()?,
            tree,
            &[&head],
            &msg,
        )?;
        commit::move_ref(
            self.repo(),
            MAIN,
            Some(head.id()),
            Some(c),
            "teahouse: write",
        )?;
        Ok(c)
    }

    /// Commits `edits` as a new proposal of ours; returns its ref and id.
    fn commit_proposal(
        &self,
        problem: &str,
        title: &str,
        reason: Option<&str>,
        edits: &[Edit],
    ) -> Result<(String, String)> {
        let base = ascii_slug(title, 30);
        let name = if base.is_empty() {
            format!("p-{}", suffix())
        } else {
            format!("{base}-{}", suffix())
        };
        let id = format!("{}/{name}", self.me.id);
        let refname = format!("{PROPOSALS}{id}");
        let head = self.head()?;
        let tree = commit::tree_with(self.repo(), Some(&head.tree()?), edits)?;
        let msg = commit::message(
            title,
            reason,
            &[("Problem", problem.to_string()), ("Act", "propose".into())],
        );
        let c = commit::signed_commit(
            self.repo(),
            &self.identity,
            &self.author()?,
            tree,
            &[&head],
            &msg,
        )?;
        commit::move_ref(self.repo(), &refname, None, Some(c), "teahouse: propose")?;
        Ok((refname, id))
    }

    /// Syncs after a local write. Refusals and conflicts undo the write and
    /// keep a draft.
    fn publish(
        &self,
        undo: Vec<Undo>,
        command: &str,
        args: &Value,
        became_proposal: bool,
        id: Option<String>,
    ) -> Result<WriteResult> {
        let link = Link {
            repo: self.repo(),
            identity: &self.identity,
            known_hosts: &self.client.known_hosts(),
            blobs: &self.client.root().join("blobs"),
        };
        let pushed = match link.sync() {
            Ok(sync::Outcome::Synced) => true,
            Ok(sync::Outcome::NoRemote) => false,
            Err(e) if matches!(e.code, Code::Offline) => false,
            Err(mut e) => {
                self.undo(&undo)?;
                let draft = self.client.save_draft(&self.house, command, args, &e)?;
                e.params.insert("draft".into(), draft);
                return Err(e);
            }
        };
        let snap = self.client.snapshot(&self.house)?;
        Ok(WriteResult {
            commit: if pushed { "pushed" } else { "local" },
            sync: snap.summary_sync()?,
            became_proposal,
            id,
        })
    }

    fn undo(&self, undo: &[Undo]) -> Result<()> {
        let repo = self.repo();
        for u in undo {
            match u {
                Undo::Main(c) => {
                    let Ok(tip) = repo.refname_to_id(MAIN) else {
                        continue;
                    };
                    // Ours is the newest commit on main unless a sync
                    // replayed it; find it by its place, not its id.
                    let ours = repo.find_commit(*c)?;
                    let parent = ours.parent_id(0)?;
                    if tip == *c {
                        commit::move_ref(repo, MAIN, Some(tip), Some(parent), "teahouse: refused")?;
                    } else if repo.graph_descendant_of(tip, *c)? {
                        let later = sync::first_parent_since(repo, tip, *c)?;
                        let (new_tip, _) =
                            sync::replay(repo, &self.identity, &later, parent, |_| true)?;
                        commit::move_ref(
                            repo,
                            MAIN,
                            Some(tip),
                            Some(new_tip),
                            "teahouse: refused",
                        )?;
                    } else {
                        // Replayed during the sync: the newest local commit
                        // with the same message is ours.
                        let tracking = repo.refname_to_id(sync::TRACKING_MAIN).ok();
                        let base = match tracking {
                            Some(t) => repo.merge_base(tip, t)?,
                            None => parent,
                        };
                        let local = sync::first_parent_since(repo, tip, base)?;
                        let msg = ours.message_bytes().to_vec();
                        let mine = local.iter().rev().find(|o| {
                            repo.find_commit(**o)
                                .is_ok_and(|x| x.message_bytes() == msg.as_slice())
                        });
                        if let Some(&m) = mine {
                            let (new_tip, _) =
                                sync::replay(repo, &self.identity, &local, base, |o| o != m)?;
                            commit::move_ref(
                                repo,
                                MAIN,
                                Some(tip),
                                Some(new_tip),
                                "teahouse: refused",
                            )?;
                        }
                    }
                }
                Undo::Proposal(name, old) => {
                    let current = repo.refname_to_id(name).ok();
                    commit::move_ref(repo, name, current, *old, "teahouse: refused")?;
                }
                Undo::PendingPurge(hash) => sync::remove_pending_purge(repo.path(), hash)?,
            }
        }
        Ok(())
    }

    // -- content commands -------------------------------------------------

    pub fn open_problem(
        self,
        title: &str,
        body: &[SectionArg],
        args: &Value,
    ) -> Result<WriteResult> {
        let title = title.trim();
        if title.is_empty() {
            return Err(invalid("a problem needs a title / 问题需要标题").with("field", "title"));
        }
        let id = self.next_problem_id();
        let path = format!("problems/{id}-{}.md", slug(title, 40));
        let edits = vec![Edit::Put(
            path,
            render_problem(title, "open", body).into_bytes(),
        )];
        let trailers = [("Problem", id.clone()), ("Act", "open-problem".to_string())];
        if self.role() == Role::Guest {
            return self.into_proposal(&id, title, None, &edits, "openProblem", args);
        }
        let c = self.commit_main(&edits, title, None, &trailers)?;
        self.publish(vec![Undo::Main(c)], "openProblem", args, false, Some(id))
    }

    fn into_proposal(
        self,
        problem: &str,
        title: &str,
        reason: Option<&str>,
        edits: &[Edit],
        command: &str,
        args: &Value,
    ) -> Result<WriteResult> {
        let (refname, id) = self.commit_proposal(problem, title, reason, edits)?;
        self.publish(
            vec![Undo::Proposal(refname, None)],
            command,
            args,
            true,
            Some(id),
        )
    }

    pub fn set_status(
        self,
        pid: &str,
        status: Status,
        owner: Option<&str>,
        args: &Value,
    ) -> Result<WriteResult> {
        self.require(&[Role::Host, Role::Steward], "setStatus")?;
        self.require_problem(pid)?;
        if let Some(o) = owner.filter(|o| !o.is_empty()) {
            if self.snap.members.by_id(o).is_none() {
                return Err(
                    invalid(format!("{o} has no seat here / {o} 不在座位表上")).with("owner", o)
                );
            }
        }
        let (_, path) = self.snap.problem_path(pid).expect("checked");
        let path = path.to_string();
        let text = self.snap.text(&path)?.unwrap_or_default();
        let status_s = serde_json::to_value(status)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();
        let mut changes: Vec<(&str, Option<&str>)> = vec![("status", Some(&status_s))];
        if let Some(o) = owner {
            changes.push(("owner", (!o.is_empty()).then_some(o)));
        }
        let new = format::set_fields(&text, &changes);
        if new == text {
            return Err(invalid("nothing to change / 没有变化"));
        }
        let title = format::parse_problem(&text, pid, "").title;
        let c = self.commit_main(
            &[Edit::Put(path, new.into_bytes())],
            &title,
            None,
            &[
                ("Problem", pid.to_string()),
                ("Act", "set-status".into()),
                ("Status", status_s.clone()),
            ],
        )?;
        self.publish(vec![Undo::Main(c)], "setStatus", args, false, None)
    }

    /// The files of a new work (and its attachment, copied into the cache).
    fn work_files(
        &self,
        pid: &str,
        kind: WorkKindArg,
        title: &str,
        text: Option<&str>,
        from_path: Option<&str>,
        note: Option<&str>,
    ) -> Result<(String, Vec<Edit>)> {
        let title = title.trim();
        if title.is_empty() {
            return Err(invalid("a work needs a title / 成果需要标题").with("field", "title"));
        }
        let dir = format!("works/{pid}/");
        let stem = slug(title, 40);
        let mut fields: Vec<(&str, &str)> = vec![("title", title)];
        match kind {
            WorkKindArg::Doc | WorkKindArg::Plan => {
                if kind == WorkKindArg::Plan {
                    fields.push(("kind", "plan"));
                }
                if let Some(n) = note {
                    fields.push(("note", n));
                }
                let path = self.free_path(&dir, &stem, ".md");
                let content = format::render_frontmatter(&fields) + text.unwrap_or("");
                Ok((path.clone(), vec![Edit::Put(path, content.into_bytes())]))
            }
            WorkKindArg::Table => {
                if let Some(n) = note {
                    fields.push(("note", n));
                }
                let path = self.free_path(&dir, &stem, ".csv");
                let meta = format::render_frontmatter(&fields);
                Ok((
                    path.clone(),
                    vec![
                        Edit::Put(path.clone(), text.unwrap_or("").as_bytes().to_vec()),
                        Edit::Put(format!("{path}.meta.md"), meta.into_bytes()),
                    ],
                ))
            }
            WorkKindArg::Image | WorkKindArg::File => {
                let from = from_path.ok_or_else(|| {
                    invalid("attachments need fromPath / 附件需要 fromPath")
                        .with("field", "fromPath")
                })?;
                let ext = std::path::Path::new(from)
                    .extension()
                    .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
                    .unwrap_or_default();
                let path = self.free_path(&dir, &stem, &format!("{ext}.ref.md"));
                let content = self.blob_ref(title, from, note)?;
                Ok((path.clone(), vec![Edit::Put(path, content.into_bytes())]))
            }
        }
    }

    /// Copies a file into the attachment cache and returns its reference.
    /// Uploading to the relay's attachment store comes with step 3.
    fn blob_ref(&self, title: &str, from: &str, note: Option<&str>) -> Result<String> {
        let meta = std::fs::metadata(from)
            .map_err(|e| invalid(format!("{from}: {e}")).with("fromPath", from))?;
        if meta.len() > MAX_BLOB {
            return Err(Error::new(
                Code::TooLarge,
                "attachments are limited to 100 MiB / 附件最大 100 MiB",
            )
            .with("limit", MAX_BLOB.to_string()));
        }
        let data = std::fs::read(from)?;
        let hash = teahouse_core::sha256::hex(&data);
        let dir = self.client.root().join("blobs");
        std::fs::create_dir_all(&dir)?;
        let dest = dir.join(&hash);
        if !dest.exists() {
            let tmp = dir.join(format!(".{hash}.tmp"));
            std::fs::write(&tmp, &data)?;
            std::fs::rename(&tmp, &dest)?;
        }
        let size = data.len().to_string();
        let blob = format!("sha256:{hash}");
        let mut fields = vec![
            ("title", title),
            ("blob", blob.as_str()),
            ("size", size.as_str()),
            ("type", mime_of(from)),
        ];
        if let Some(n) = note {
            fields.push(("note", n));
        }
        Ok(format::render_frontmatter(&fields))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_work(
        self,
        pid: &str,
        kind: WorkKindArg,
        title: &str,
        text: Option<&str>,
        from_path: Option<&str>,
        note: Option<&str>,
        args: &Value,
    ) -> Result<WriteResult> {
        self.require_problem(pid)?;
        let (path, edits) = self.work_files(pid, kind, title, text, from_path, note)?;
        if self.role() == Role::Guest {
            return self.into_proposal(pid, title.trim(), None, &edits, "addWork", args);
        }
        let c = self.commit_main(
            &edits,
            title.trim(),
            None,
            &[("Problem", pid.to_string()), ("Act", "add-work".into())],
        )?;
        self.publish(vec![Undo::Main(c)], "addWork", args, false, Some(path))
    }

    /// The edits that replace a work's content, keeping its title.
    fn work_update(
        &self,
        id: &str,
        text: Option<&str>,
        from_path: Option<&str>,
    ) -> Result<(String, String, Vec<Edit>)> {
        let Some((pid, _)) = format::parse_work_path(id) else {
            return Err(Error::not_found(format!("{id} is not a work / 不是成果")).with("id", id));
        };
        let old = self.snap.text(id)?.ok_or_else(|| {
            Error::not_found(format!("no work {id} / 没有这个成果")).with("id", id)
        })?;
        let (fm, body) = format::split_frontmatter(&old);
        let title = fm
            .get("title")
            .map(str::to_string)
            .unwrap_or_else(|| format::file_title(id));
        let kind = format::classify_work(id, Some(&fm));
        let new = match kind {
            format::WorkFile::Work(WorkKind::Doc | WorkKind::Plan) => {
                let head = &old[..old.len() - body.len()];
                format!(
                    "{head}{}",
                    text.ok_or_else(|| invalid("text is required / 需要 text"))?
                )
            }
            format::WorkFile::Work(WorkKind::Table) => text
                .ok_or_else(|| invalid("text is required / 需要 text"))?
                .to_string(),
            format::WorkFile::Work(WorkKind::Image | WorkKind::File) if id.ends_with(".ref.md") => {
                let from =
                    from_path.ok_or_else(|| invalid("fromPath is required / 需要 fromPath"))?;
                self.blob_ref(&title, from, fm.get("note"))?
            }
            _ => {
                return Err(invalid(format!(
                    "{id} cannot be updated here / 这个文件不能在这里更新"
                ))
                .with("id", id))
            }
        };
        if new == old {
            return Err(invalid("nothing to change / 没有变化"));
        }
        Ok((
            pid.to_string(),
            title,
            vec![Edit::Put(id.to_string(), new.into_bytes())],
        ))
    }

    pub fn update_work(
        self,
        id: &str,
        text: Option<&str>,
        from_path: Option<&str>,
        args: &Value,
    ) -> Result<WriteResult> {
        let (pid, title, edits) = self.work_update(id, text, from_path)?;
        if self.role() == Role::Guest {
            return self.into_proposal(&pid, &title, None, &edits, "updateWork", args);
        }
        let c = self.commit_main(
            &edits,
            &title,
            None,
            &[("Problem", pid), ("Act", "update-work".into())],
        )?;
        self.publish(
            vec![Undo::Main(c)],
            "updateWork",
            args,
            false,
            Some(id.to_string()),
        )
    }

    pub fn note(
        self,
        problem: Option<&str>,
        notice: Option<&str>,
        text: &str,
        args: &Value,
    ) -> Result<WriteResult> {
        if text.trim().is_empty() {
            return Err(invalid("a post needs text / 留言不能为空").with("field", "text"));
        }
        check_text(text.as_bytes()).map_err(|m| {
            let code = if text.len() > rules::MAX_TEXT_BYTES {
                Code::TooLarge
            } else {
                Code::Invalid
            };
            Error::new(code, m).with("limit", rules::MAX_TEXT_BYTES.to_string())
        })?;
        let (dir, topic) = match (problem, notice) {
            (Some(p), None) => {
                self.require_problem(p)?;
                (format!("talk/{p}/"), p.to_string())
            }
            (None, Some(n)) => {
                if self.snap.entry(&format!("board/{n}.md")).is_none() {
                    return Err(
                        Error::not_found(format!("no notice {n} / 没有这条公告")).with("notice", n)
                    );
                }
                (format!("talk/board/{n}/"), "board".to_string())
            }
            _ => {
                return Err(invalid(
                    "give either problem or notice / problem 和 notice 只能给一个",
                ))
            }
        };
        // One post per second per member; a second one waits a second.
        let mut at = sync::now();
        let path = loop {
            let p = format!("{dir}{}-{}.md", format_stamp(at), self.me.id);
            if self.snap.entry(&p).is_none() {
                break p;
            }
            at += 1;
        };
        let c = self.commit_main(
            &[Edit::Put(path.clone(), text.as_bytes().to_vec())],
            &first_line(text, 40),
            None,
            &[("Problem", topic), ("Act", "note".into())],
        )?;
        self.publish(vec![Undo::Main(c)], "note", args, false, Some(path))
    }

    pub fn remove_note(self, id: &str, args: &Value) -> Result<WriteResult> {
        self.require(&[Role::Host, Role::Steward], "removeNote")?;
        let Some(post) = rules::parse_talk_path(id).filter(|_| self.snap.entry(id).is_some())
        else {
            return Err(Error::not_found(format!("no post {id} / 没有这条留言")).with("id", id));
        };
        let topic = match post.topic {
            rules::TalkTopic::Problem(p) => p.to_string(),
            rules::TalkTopic::Board(_) => "board".into(),
        };
        let c = self.commit_main(
            &[Edit::Remove(id.to_string())],
            "删除留言",
            None,
            &[("Problem", topic), ("Act", "remove-note".into())],
        )?;
        self.publish(vec![Undo::Main(c)], "removeNote", args, false, None)
    }

    pub fn post_notice(self, title: &str, text: &str, args: &Value) -> Result<WriteResult> {
        let title = title.trim();
        if title.is_empty() {
            return Err(invalid("a notice needs a title / 公告需要标题").with("field", "title"));
        }
        let content = format::render_frontmatter(&[("title", title)]) + text;
        check_text(content.as_bytes()).map_err(|m| {
            let code = if content.len() > rules::MAX_TEXT_BYTES {
                Code::TooLarge
            } else {
                Code::Invalid
            };
            Error::new(code, m).with("limit", rules::MAX_TEXT_BYTES.to_string())
        })?;
        let stem = match ascii_slug(title, 40) {
            s if s.is_empty() => format!("n{}", format_stamp(sync::now()).to_lowercase()),
            s => s,
        };
        let path = self.free_path("board/", &stem, ".md");
        let name = format::parse_notice_path(&path)
            .unwrap_or_default()
            .to_string();
        let edits = vec![Edit::Put(path, content.into_bytes())];
        if self.role() == Role::Guest {
            return self.into_proposal("board", title, None, &edits, "postNotice", args);
        }
        let c = self.commit_main(
            &edits,
            title,
            None,
            &[("Problem", "board".into()), ("Act", "notice".into())],
        )?;
        self.publish(vec![Undo::Main(c)], "postNotice", args, false, Some(name))
    }

    // -- proposals --------------------------------------------------------

    pub fn propose(
        self,
        problem: &str,
        title: &str,
        reason: Option<&str>,
        changes: &[ChangeArg],
        args: &Value,
    ) -> Result<WriteResult> {
        if title.trim().is_empty() {
            return Err(invalid("a proposal needs a title / 提案需要标题").with("field", "title"));
        }
        if changes.is_empty() {
            return Err(invalid("a proposal needs changes / 提案需要改动").with("field", "changes"));
        }
        if problem != "board" {
            self.require_problem(problem)?;
        }
        let mut edits = Vec::new();
        for ch in changes {
            match (&ch.id, ch.delete) {
                (Some(id), true) => {
                    if self.snap.entry(id).is_none() {
                        return Err(
                            Error::not_found(format!("no file {id} / 没有这个文件")).with("id", id)
                        );
                    }
                    edits.push(Edit::Remove(id.clone()));
                    if id.ends_with(".csv") && self.snap.entry(&format!("{id}.meta.md")).is_some() {
                        edits.push(Edit::Remove(format!("{id}.meta.md")));
                    }
                }
                (Some(id), false) => {
                    let (_, _, e) =
                        self.work_update(id, ch.text.as_deref(), ch.from_path.as_deref())?;
                    edits.extend(e);
                }
                (None, _) => {
                    let kind = ch
                        .kind
                        .ok_or_else(|| invalid("a new file needs a kind / 新文件需要 kind"))?;
                    let title = ch.title.as_deref().unwrap_or("");
                    let (_, e) = self.work_files(
                        problem,
                        kind,
                        title,
                        ch.text.as_deref(),
                        ch.from_path.as_deref(),
                        None,
                    )?;
                    edits.extend(e);
                }
            }
        }
        let (refname, id) = self.commit_proposal(problem, title.trim(), reason, &edits)?;
        self.publish(
            vec![Undo::Proposal(refname, None)],
            "propose",
            args,
            false,
            Some(id),
        )
    }

    fn find_proposal(&self, id: &str) -> Result<(String, Oid, crate::model::Proposal)> {
        let refname = format!("{PROPOSALS}{id}");
        let tip = self.repo().refname_to_id(&refname).map_err(|_| {
            Error::not_found(format!("no open proposal {id} / 没有这个提案")).with("proposal", id)
        })?;
        let p = self
            .snap
            .proposals(false)?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| {
                Error::not_found(format!("no open proposal {id} / 没有这个提案"))
                    .with("proposal", id)
            })?;
        Ok((refname, tip, p))
    }

    pub fn withdraw(self, id: &str, args: &Value) -> Result<WriteResult> {
        let (refname, tip, p) = self.find_proposal(id)?;
        if p.by != self.me.id {
            return Err(forbidden("withdraw"));
        }
        commit::move_ref(self.repo(), &refname, Some(tip), None, "teahouse: withdraw")?;
        self.publish(
            vec![Undo::Proposal(refname, Some(tip))],
            "withdraw",
            args,
            false,
            None,
        )
    }

    pub fn adopt(self, id: &str, args: &Value) -> Result<WriteResult> {
        self.require(&[Role::Host], "adopt")?;
        let (refname, tip, p) = self.find_proposal(id)?;
        let repo = self.repo();
        let head = self.head()?;
        let theirs = repo.find_commit(tip)?;
        let mut index = repo.merge_commits(&head, &theirs, None)?;
        if index.has_conflicts() {
            return Err(Error::new(
                Code::Conflict,
                "the proposal conflicts with what is on main now / 提案和现在的内容冲突",
            )
            .with("proposal", id));
        }
        let mut tree = index.write_tree_to(repo)?;
        // A new problem whose number main took meanwhile moves to the next
        // free number, inside this merge (CLIENT-API.md §1.5).
        let mut problem = p.problem.clone();
        let mut renumbered = Vec::new();
        let base = repo.merge_base(head.id(), tip)?;
        let base_tree = repo.find_commit(base)?.tree()?;
        let theirs_tree = theirs.tree()?;
        let diff = repo.diff_tree_to_tree(Some(&base_tree), Some(&theirs_tree), None)?;
        let head_tree = head.tree()?;
        let on_main = sync::problem_ids(repo, &head_tree)?;
        let mut ids = on_main.clone();
        let mut map = std::collections::HashMap::new();
        for d in diff.deltas() {
            let path =
                String::from_utf8_lossy(d.new_file().path_bytes().unwrap_or_default()).into_owned();
            if let (git2::Delta::Added, Some((pid, _))) =
                (d.status(), format::parse_problem_path(&path))
            {
                if on_main.contains(pid) && !map.contains_key(pid) {
                    let n = sync::next_id(&ids);
                    ids.insert(n.clone());
                    map.insert(pid.to_string(), n.clone());
                    renumbered.push((pid.to_string(), n));
                }
            }
        }
        if !map.is_empty() {
            // Move what the proposal added under the taken number; what main
            // already had there stays.
            let mut b = git2::build::TreeUpdateBuilder::new();
            for d in diff.deltas() {
                if d.status() != git2::Delta::Added {
                    continue;
                }
                let path = String::from_utf8_lossy(d.new_file().path_bytes().unwrap_or_default())
                    .into_owned();
                let moved = sync::renumber_path(&path, &map);
                if moved != path {
                    b.upsert(moved.as_str(), d.new_file().id(), d.new_file().mode());
                    if head_tree.get_path(std::path::Path::new(&path)).is_err() {
                        b.remove(path.as_str());
                    }
                }
            }
            tree = b.create_updated(repo, &repo.find_tree(tree)?)?;
            if let Some(n) = map.get(&problem) {
                problem = n.clone();
            }
        }
        let merged_tree = repo.find_tree(tree)?;
        let exists = problem == "board" || problem_file_exists(repo, &merged_tree, &problem)?;
        let problem = if exists { problem } else { "meta".to_string() };
        let mut trailers = vec![
            ("Problem", problem),
            ("Act", "adopt".to_string()),
            ("Proposal", id.to_string()),
        ];
        for (a, b) in &renumbered {
            trailers.push(("Renumbered", format!("{a} -> {b}")));
        }
        let msg = commit::message(&format!("采纳提案：{}", p.title), None, &trailers);
        let c = commit::signed_commit(
            repo,
            &self.identity,
            &self.author()?,
            tree,
            &[&head, &theirs],
            &msg,
        )?;
        commit::move_ref(repo, MAIN, Some(head.id()), Some(c), "teahouse: adopt")?;
        commit::move_ref(repo, &refname, Some(tip), None, "teahouse: adopt")?;
        self.publish(
            vec![Undo::Main(c), Undo::Proposal(refname, Some(tip))],
            "adopt",
            args,
            false,
            None,
        )
    }

    pub fn decline(self, id: &str, reason: Option<&str>, args: &Value) -> Result<WriteResult> {
        self.require(&[Role::Host], "decline")?;
        let (refname, tip, p) = self.find_proposal(id)?;
        let problem = if p.problem == "board" || self.problem_exists(&p.problem) {
            p.problem.clone()
        } else {
            "meta".into()
        };
        let c = self.commit_main(
            &[],
            &format!("不采纳提案：{}", p.title),
            reason,
            &[
                ("Problem", problem),
                ("Act", "decline".into()),
                ("Proposal", id.to_string()),
            ],
        )?;
        commit::move_ref(self.repo(), &refname, Some(tip), None, "teahouse: decline")?;
        self.publish(
            vec![Undo::Main(c), Undo::Proposal(refname, Some(tip))],
            "decline",
            args,
            false,
            None,
        )
    }

    /// The host removes an attachment's content for good: the reference is
    /// marked `purged: <time> <host>` (a signed commit), and once that is on
    /// the relay, the relay deletes the file (CLIENT-API.md §1.6).
    pub fn purge_blob(self, id: &str, args: &Value) -> Result<WriteResult> {
        self.require(&[Role::Host], "purgeBlob")?;
        let Some((pid, _)) = format::parse_work_path(id).filter(|_| id.ends_with(".ref.md")) else {
            return Err(
                Error::not_found(format!("{id} is not an attachment / 不是附件")).with("id", id),
            );
        };
        let text = self.snap.text(id)?.ok_or_else(|| {
            Error::not_found(format!("{id} is not an attachment / 不是附件")).with("id", id)
        })?;
        let r = format::parse_blob_ref(&text).ok_or_else(|| {
            invalid(format!("{id}: malformed attachment reference")).with("id", id)
        })?;
        if r.purged {
            return Err(
                invalid("this attachment was already removed / 这个附件已经删除了").with("id", id),
            );
        }
        let stamp = format!(
            "{} {}",
            teahouse_core::rules::format_rfc3339(sync::now()),
            self.me.id
        );
        let new = format::set_fields(&text, &[("purged", Some(&stamp))]);
        let title = r.title.clone().unwrap_or_else(|| format::file_title(id));
        let pid = pid.to_string();
        let c = self.commit_main(
            &[Edit::Put(id.to_string(), new.into_bytes())],
            &format!("删除附件：{title}"),
            None,
            &[("Problem", pid), ("Act", "update-work".into())],
        )?;
        sync::add_pending_purge(self.repo().path(), &r.hash)?;
        let result = self.publish(
            vec![Undo::Main(c), Undo::PendingPurge(r.hash.clone())],
            "purgeBlob",
            args,
            false,
            Some(id.to_string()),
        )?;
        let _ = std::fs::remove_file(self.client.root().join("blobs").join(&r.hash));
        Ok(result)
    }

    // -- seats ------------------------------------------------------------

    pub fn seat(
        self,
        remove: Option<&str>,
        role: Option<(&str, &str)>,
        args: &Value,
    ) -> Result<WriteResult> {
        self.require(&[Role::Host], "seat")?;
        let text = self.snap.text(MEMBERS_PATH)?.unwrap_or_default();
        let mut m = Members::parse(&text).map_err(|e| Error::internal(e.to_string()))?;
        let mut subject = Vec::new();
        if let Some(id) = remove {
            let before = m.members.len();
            m.members.retain(|x| x.id != id);
            if m.members.len() == before {
                return Err(
                    Error::not_found(format!("{id} has no seat / {id} 不在座位表上"))
                        .with("member", id),
                );
            }
            subject.push(format!("{id} 离座"));
        }
        if let Some((id, r)) = role {
            let r = Role::parse(r)
                .ok_or_else(|| invalid(format!("unknown role {r}")).with("role", r))?;
            let x = m.members.iter_mut().find(|x| x.id == id).ok_or_else(|| {
                Error::not_found(format!("{id} has no seat / {id} 不在座位表上")).with("member", id)
            })?;
            x.role = r;
            subject.push(format!("{id} 改为 {r}"));
        }
        if subject.is_empty() {
            return Err(invalid("give remove or role / 需要 remove 或 role"));
        }
        let toml = m.to_toml().map_err(|e| invalid(e.to_string()))?;
        let c = self.commit_main(
            &[Edit::Put(MEMBERS_PATH.to_string(), toml.into_bytes())],
            &format!("座位变更：{}", subject.join("，")),
            None,
            &[("Problem", "meta".into()), ("Act", "seat".into())],
        )?;
        self.publish(vec![Undo::Main(c)], "seat", args, false, None)
    }
}

fn problem_file_exists(repo: &Repository, tree: &git2::Tree<'_>, id: &str) -> Result<bool> {
    Ok(sync::problem_ids(repo, tree)?.contains(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs() {
        assert_eq!(slug("小区停车位 不好找", 40), "小区停车位-不好找");
        assert_eq!(slug("a/b\\\\c:d*e", 40), "abcde");
        assert_eq!(slug("..", 40), "untitled");
        assert_eq!(slug("一二三四五", 3), "一二三");
        assert_eq!(ascii_slug("Night parking: 2h!", 30), "night-parking-2h");
        assert_eq!(ascii_slug("夜间临停", 30), "");
        assert_eq!(first_line("\n  第一行很长很长\n第二行", 4), "第一行很…");
        assert_eq!(suffix().len(), 4);
    }

    #[test]
    fn problems_render() {
        let body = [
            SectionArg {
                heading: "".into(),
                text: "摘要。".into(),
            },
            SectionArg {
                heading: "现状".into(),
                text: "42 个车位".into(),
            },
        ];
        let t = render_problem("修 #3 号楼", "open", &body);
        assert_eq!(
            t,
            "---\ntitle: \"修 #3 号楼\"\nstatus: open\n---\n摘要。\n\n## 现状\n42 个车位\n"
        );
        let p = format::parse_problem(&t, "0001", "");
        assert_eq!(p.title, "修 #3 号楼");
        assert_eq!(format::summary(&p.body), "摘要。");
    }
}
