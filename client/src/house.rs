//! A snapshot of one teahouse's local clone, and every read view built from
//! it (CLIENT-API.md §4 "读取"). Reads only committed objects of the local
//! `main` and `refs/proposals/*`; never the network, never the working tree.

use crate::error::{Code, Error, Result};
use crate::format::{self, WorkFile};
use crate::git::{Git, Step, TreeFile};
use crate::model::*;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use teahouse_core::rules::{format_rfc3339, TalkTopic};
use teahouse_core::{members::valid_member_id, Members, PublicKey, MAX_MEMBERS, MEMBERS_PATH};

/// How many events `House.recent` carries.
const RECENT: usize = 30;
/// Text files larger than this are not inlined or diffed.
const MAX_INLINE: u64 = 1024 * 1024;

const PROPOSALS: &str = "refs/proposals/";
const UPSTREAM: &str = "refs/remotes/origin/main";

pub struct Snapshot {
    pub id: String,
    pub git: Git,
    blobs_dir: PathBuf,
    pub members: Members,
    /// The seat whose key is this device's, if any.
    pub me: Option<teahouse_core::Member>,
    /// The commit `main` points at.
    head: String,
    steps: Vec<Step>,
    tree: BTreeMap<String, TreeFile>,
    /// Path → the step that last added it.
    added: HashMap<String, usize>,
    /// Path → the step that last changed it.
    touched: HashMap<String, usize>,
    /// Post path → its place in the order posts entered `main`.
    post_order: HashMap<String, usize>,
    /// Member id → the step that first seated them.
    joined: HashMap<String, usize>,
}

fn when(t: i64) -> String {
    format_rfc3339(t)
}

fn text_of(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

impl Snapshot {
    pub fn load(
        id: &str,
        dir: &Path,
        blobs_dir: &Path,
        device: Option<&PublicKey>,
    ) -> Result<Snapshot> {
        let git = Git::open(dir)?;
        let Some(head) = git.resolve("refs/heads/main")? else {
            return Err(Error::not_found(format!(
                "teahouse {id} has no main yet / 茶馆 {id} 还没有内容"
            ))
            .with("house", id));
        };
        let tree: BTreeMap<String, TreeFile> = git
            .ls_tree(&head)?
            .into_iter()
            .map(|f| (f.path.clone(), f))
            .collect();
        let chart = match tree.get(MEMBERS_PATH) {
            Some(f) => git
                .read_blobs(std::slice::from_ref(&f.oid))?
                .pop()
                .flatten(),
            None => None,
        };
        let members = chart
            .as_deref()
            .ok_or_else(|| Error::internal(format!("{id}: {MEMBERS_PATH} is missing")))
            .and_then(|b| {
                Members::parse(&text_of(b)).map_err(|e| Error::internal(format!("{id}: {e}")))
            })?;
        let me = device.and_then(|k| members.by_key(k)).cloned();
        let steps = git.first_parent_steps(&head)?;

        let mut added = HashMap::new();
        let mut touched = HashMap::new();
        let mut post_order = HashMap::new();
        let mut chart_steps = Vec::new();
        for (i, s) in steps.iter().enumerate() {
            let mut new_posts = Vec::new();
            for (c, p) in &s.changes {
                touched.insert(p.clone(), i);
                if *c == 'A' {
                    added.insert(p.clone(), i);
                    if p.starts_with("talk/") {
                        new_posts.push(p.as_str());
                    }
                }
                if p == MEMBERS_PATH && *c != 'D' {
                    chart_steps.push(i);
                }
            }
            // Within one step, posts go by file name (RELAY.md "显示顺序").
            new_posts.sort_unstable();
            for p in new_posts {
                let n = post_order.len();
                post_order.insert(p.to_string(), n);
            }
        }

        let mut joined = HashMap::new();
        for &i in &chart_steps {
            let chart = git.read_path(&steps[i].hash, MEMBERS_PATH)?;
            if let Some(m) = chart.and_then(|b| Members::parse(&text_of(&b)).ok()) {
                for m in m.members {
                    joined.entry(m.id).or_insert(i);
                }
            }
        }

        Ok(Snapshot {
            id: id.to_string(),
            git,
            blobs_dir: blobs_dir.to_path_buf(),
            members,
            me,
            head,
            steps,
            tree,
            added,
            touched,
            post_order,
            joined,
        })
    }

    // -- small helpers ------------------------------------------------------

    /// The commit `main` points at.
    pub(crate) fn head(&self) -> &str {
        &self.head
    }

    /// A file of `main`, by path.
    pub(crate) fn entry(&self, path: &str) -> Option<&TreeFile> {
        self.tree.get(path)
    }

    /// The text of a file of `main`.
    pub(crate) fn text(&self, path: &str) -> Result<Option<String>> {
        Ok(self.read_many(&[path])?.remove(path).map(|b| text_of(&b)))
    }

    pub(crate) fn me_id(&self) -> String {
        self.me.as_ref().map(|m| m.id.clone()).unwrap_or_default()
    }

    pub(crate) fn my_role(&self) -> &'static str {
        // A device without a seat reads as a guest would (CLIENT-API.md §2).
        self.me.as_ref().map_or("guest", |m| m.role.as_str())
    }

    /// The member id a step's author stands for, or the author's name.
    fn by(&self, s: &Step) -> String {
        if let Some(id) = format::author_member(&self.members, &s.author_name, &s.author_email) {
            return id.to_string();
        }
        let local = s.author_email.split('@').next().unwrap_or("");
        if valid_member_id(local) {
            local.to_string()
        } else {
            s.author_name.clone()
        }
    }

    fn event(&self, s: &Step) -> Event {
        let problem = s
            .problem
            .first()
            .filter(|p| p.len() == 4 && p.bytes().all(|b| b.is_ascii_digit()))
            .cloned();
        Event {
            by: self.by(s),
            act: format::infer_act(s.act.first().map(String::as_str), &s.changes),
            title: s.subject.clone(),
            when: when(s.time),
            problem,
            status: s.status.first().and_then(|v| Status::parse(v)),
        }
    }

    fn added_step(&self, path: &str) -> Option<&Step> {
        self.added.get(path).map(|&i| &self.steps[i])
    }

    fn touched_step(&self, path: &str) -> Option<&Step> {
        self.touched.get(path).map(|&i| &self.steps[i])
    }

    pub(crate) fn read_many(&self, paths: &[&str]) -> Result<HashMap<String, Vec<u8>>> {
        let specs: Vec<String> = paths
            .iter()
            .filter_map(|p| self.tree.get(*p))
            .map(|f| f.oid.clone())
            .collect();
        let got = self.git.read_blobs(&specs)?;
        let mut by_oid: HashMap<&str, Vec<u8>> = HashMap::new();
        for (spec, b) in specs.iter().zip(got) {
            if let Some(b) = b {
                by_oid.insert(spec.as_str(), b);
            }
        }
        Ok(paths
            .iter()
            .filter_map(|p| {
                let f = self.tree.get(*p)?;
                Some((p.to_string(), by_oid.get(f.oid.as_str())?.clone()))
            })
            .collect())
    }

    pub(crate) fn files_under(&self, prefix: &str) -> impl Iterator<Item = &TreeFile> + '_ {
        let prefix = prefix.to_string();
        self.tree
            .range(prefix.clone()..)
            .take_while(move |(p, _)| p.starts_with(&prefix))
            .map(|(_, f)| f)
    }

    pub(crate) fn problem_files(&self) -> Vec<(&str, &str, &str)> {
        self.files_under("problems/")
            .filter_map(|f| {
                format::parse_problem_path(&f.path).map(|(id, slug)| (id, slug, f.path.as_str()))
            })
            .collect()
    }

    pub(crate) fn problem_path(&self, pid: &str) -> Option<(&str, &str)> {
        self.problem_files()
            .into_iter()
            .find(|(id, _, _)| *id == pid)
            .map(|(_, slug, path)| (slug, path))
    }

    /// Steps that concern a problem: by trailer, or by the paths they change.
    fn problem_steps(&self, pid: &str) -> Vec<&Step> {
        let works = format!("works/{pid}/");
        let talk = format!("talk/{pid}/");
        self.steps
            .iter()
            .filter(|s| {
                s.problem.iter().any(|p| p == pid)
                    || s.changes.iter().any(|(_, p)| {
                        p.starts_with(&works)
                            || p.starts_with(&talk)
                            || format::parse_problem_path(p).is_some_and(|(id, _)| id == pid)
                    })
            })
            .collect()
    }

    fn notes_under(&self, prefix: &str, texts: &HashMap<String, Vec<u8>>) -> Vec<Note> {
        let mut posts: Vec<(&str, usize)> = self
            .files_under(prefix)
            .filter(|f| !f.path[prefix.len()..].contains('/'))
            .filter_map(|f| Some((f.path.as_str(), *self.post_order.get(&f.path)?)))
            .collect();
        posts.sort_by_key(|&(_, n)| n);
        posts
            .into_iter()
            .filter_map(|(p, _)| {
                let pp = format::parse_post_path(p)?;
                Some(Note {
                    id: p.to_string(),
                    by: pp.member.to_string(),
                    text: texts.get(p).map(|b| text_of(b)).unwrap_or_default(),
                    when: when(self.added_step(p)?.time),
                    written_at: when(pp.written),
                })
            })
            .collect()
    }

    /// Where this clone stands against the relay: commits not yet pushed,
    /// and how the last sync attempt went.
    pub(crate) fn summary_sync(&self) -> Result<SyncState> {
        let record = crate::sync::Record::load(self.git.dir());
        let last_sync = record.last_ok.map(when);
        let (unpushed, behind) = match self.git.resolve(UPSTREAM)? {
            Some(upstream) => (
                self.git.count(&upstream, &self.head)?,
                self.git.count(&self.head, &upstream)?,
            ),
            // Never synced (or made locally): everything is local.
            None => (self.steps.len(), 0),
        };
        let state = match record.failed.as_deref() {
            Some("offline") => SyncKind::Offline,
            Some(_) => SyncKind::Error,
            None if unpushed > 0 => SyncKind::Ahead,
            None if behind > 0 => SyncKind::Behind,
            None => SyncKind::Synced,
        };
        Ok(SyncState {
            state,
            unpushed,
            last_sync,
            message: record.message,
        })
    }

    // -- proposals ----------------------------------------------------------

    /// Open proposals: proposal refs with commits not yet on `main`.
    pub fn proposals(&self, with_files: bool) -> Result<Vec<Proposal>> {
        let mut out = Vec::new();
        for (refname, tip) in self.git.refs(PROPOSALS)? {
            let id = refname[PROPOSALS.len()..].to_string();
            let Some((author, _)) = id.split_once('/') else {
                continue;
            };
            let commits = self.git.rev_list_reverse(&self.head, &tip)?;
            let Some(first) = commits.first() else {
                continue; // already on main
            };
            let head = self.git.commit_info(first)?;
            let last = self.git.commit_info(&tip)?;
            let reason = format::strip_trailers(&head.body);
            let files = if with_files {
                let base = self.git.merge_base(&self.head, &tip)?;
                self.file_changes(base.as_deref(), &tip)?
            } else {
                Vec::new()
            };
            out.push(Proposal {
                id: id.clone(),
                title: head.subject,
                by: author.to_string(),
                problem: head.problem.first().cloned().unwrap_or_default(),
                when: when(last.time),
                reason: (!reason.is_empty()).then_some(reason),
                files,
            });
        }
        out.sort_by(|a, b| a.when.cmp(&b.when).then(a.id.cmp(&b.id)));
        Ok(out)
    }

    fn file_changes(&self, base: Option<&str>, tip: &str) -> Result<Vec<FileChange>> {
        let mut out = Vec::new();
        for (c, path) in self.git.diff_paths(base, tip)? {
            let change = match c {
                'A' => Change::Add,
                'D' => Change::Delete,
                _ => Change::Modify,
            };
            let content = match (change, base) {
                (Change::Delete, Some(b)) => self.git.read_path(b, &path)?,
                (Change::Delete, None) => None,
                _ => self.git.read_path(tip, &path)?,
            }
            .unwrap_or_default();
            let text = text_of(&content);
            let fm = format::split_frontmatter(&text).0;
            let kind = match format::classify_work(&path, Some(&fm)) {
                WorkFile::Work(k) => k,
                WorkFile::TableMeta => WorkKind::File,
            };
            let lines = if content.len() as u64 <= MAX_INLINE {
                self.git
                    .line_diff(base, tip, &path)?
                    .map(|v| v.into_iter().map(diff_line).collect())
            } else {
                None
            };
            out.push(FileChange {
                title: fm
                    .get("title")
                    .map(str::to_string)
                    .unwrap_or_else(|| format::file_title(&path)),
                path,
                kind,
                change,
                lines,
            });
        }
        Ok(out)
    }

    /// Proposals adopted or declined, newest first, from `main`'s history.
    fn decided(&self) -> Result<Vec<Decided>> {
        let mut out = Vec::new();
        for s in self.steps.iter().rev() {
            let act = s.act.first().and_then(|a| Act::parse(a));
            let result = match act {
                Some(Act::Adopt) => Outcome::Adopted,
                Some(Act::Decline) => Outcome::Declined,
                _ => continue,
            };
            let Some(id) = s.proposal.first() else {
                continue;
            };
            let by = id
                .split_once('/')
                .map_or(id.as_str(), |(a, _)| a)
                .to_string();
            let reason = match result {
                Outcome::Declined => format::strip_trailers(&self.git.commit_info(&s.hash)?.body),
                // The proposal's own reason: the first commit it brought in.
                Outcome::Adopted => match s.parents.as_slice() {
                    [p1, p2] => match self.git.rev_list_reverse(p1, p2)?.first() {
                        Some(c) => format::strip_trailers(&self.git.commit_info(c)?.body),
                        None => String::new(),
                    },
                    _ => String::new(),
                },
            };
            let title = ["采纳提案：", "不采纳提案：", "采纳提案:", "不采纳提案:"]
                .iter()
                .find_map(|p| s.subject.strip_prefix(p))
                .unwrap_or(&s.subject)
                .trim()
                .to_string();
            out.push(Decided {
                id: id.clone(),
                title,
                by,
                result,
                when: when(s.time),
                reason: (!reason.is_empty()).then_some(reason),
            });
        }
        Ok(out)
    }

    // -- views --------------------------------------------------------------

    pub fn summary(&self) -> Result<HouseSummary> {
        Ok(HouseSummary {
            id: self.id.clone(),
            name: self.members.name.clone(),
            me: self.me_id(),
            my_role: self.my_role(),
            seats: self.members.members.len(),
            pending_proposals: self.proposals(false)?.len(),
            sync: self.summary_sync()?,
        })
    }

    fn problem_summary(
        &self,
        id: &str,
        path: &str,
        doc: &format::ProblemDoc,
        proposals: &[Proposal],
    ) -> ProblemSummary {
        let opened = self.added_step(path);
        let updated = self.problem_steps(id).last().map(|s| s.time);
        let works = format!("works/{id}/");
        ProblemSummary {
            id: id.to_string(),
            title: doc.title.clone(),
            summary: format::summary(&doc.body),
            status: doc.status,
            by: opened.map(|s| self.by(s)).unwrap_or_default(),
            owner: doc.owner.clone(),
            opened: opened.map(|s| when(s.time)).unwrap_or_default(),
            updated: updated
                .or(opened.map(|s| s.time))
                .map(when)
                .unwrap_or_default(),
            work_count: self
                .files_under(&works)
                .filter(|f| format::classify_work(&f.path, None) != WorkFile::TableMeta)
                .count(),
            pending_proposals: proposals.iter().filter(|p| p.problem == id).count(),
        }
    }

    pub fn house(&self) -> Result<House> {
        let proposals = self.proposals(true)?;
        let problems = self.problem_files();
        let notice_paths: Vec<&str> = self
            .files_under("board/")
            .filter(|f| format::parse_notice_path(&f.path).is_some())
            .map(|f| f.path.as_str())
            .collect();
        let reply_paths: Vec<&str> = self
            .files_under("talk/board/")
            .map(|f| f.path.as_str())
            .collect();
        let mut wanted: Vec<&str> = problems.iter().map(|(_, _, p)| *p).collect();
        wanted.extend(&notice_paths);
        wanted.extend(&reply_paths);
        let texts = self.read_many(&wanted)?;

        let problems = problems
            .iter()
            .map(|(id, slug, path)| {
                let doc = format::parse_problem(
                    &texts.get(*path).map(|b| text_of(b)).unwrap_or_default(),
                    id,
                    slug,
                );
                self.problem_summary(id, path, &doc, &proposals)
            })
            .collect();

        let board = notice_paths
            .iter()
            .map(|path| {
                let name = format::parse_notice_path(path).unwrap_or_default();
                let text = texts.get(*path).map(|b| text_of(b)).unwrap_or_default();
                let (fm, body) = format::split_frontmatter(&text);
                let step = self.added_step(path);
                Notice {
                    id: name.to_string(),
                    by: step.map(|s| self.by(s)).unwrap_or_default(),
                    title: fm.get("title").unwrap_or(name).to_string(),
                    text: body.trim().to_string(),
                    when: step.map(|s| when(s.time)).unwrap_or_default(),
                    replies: self.notes_under(&format!("talk/board/{name}/"), &texts),
                }
            })
            .collect();

        let members = self
            .members
            .members
            .iter()
            .map(|m| Member {
                id: m.id.clone(),
                name: m.name.clone(),
                role: m.role.as_str(),
                joined: self
                    .joined
                    .get(&m.id)
                    .map(|&i| when(self.steps[i].time))
                    .unwrap_or_default(),
            })
            .collect();

        Ok(House {
            id: self.id.clone(),
            name: self.members.name.clone(),
            signboard: self.members.signboard.clone(),
            founded: self.steps.first().map(|s| when(s.time)).unwrap_or_default(),
            capacity: MAX_MEMBERS,
            me: self.me_id(),
            members,
            problems,
            recent: self
                .steps
                .iter()
                .rev()
                .take(RECENT)
                .map(|s| self.event(s))
                .collect(),
            proposals,
            decided: self.decided()?,
            board,
            sync: self.summary_sync()?,
        })
    }

    pub fn problem(&self, pid: &str) -> Result<crate::model::Problem> {
        let Some((slug, path)) = self.problem_path(pid) else {
            return Err(
                Error::not_found(format!("no problem {pid} / 没有问题 {pid}")).with("problem", pid),
            );
        };
        let works_prefix = format!("works/{pid}/");
        let talk_prefix = format!("talk/{pid}/");
        let mut wanted: Vec<&str> = vec![path];
        wanted.extend(self.files_under(&works_prefix).map(|f| f.path.as_str()));
        wanted.extend(self.files_under(&talk_prefix).map(|f| f.path.as_str()));
        // Only text-like files are read in full; binaries are listed by size.
        wanted.retain(|p| {
            self.tree.get(*p).and_then(|f| f.size).unwrap_or(0) <= MAX_INLINE
                && (p.ends_with(".md") || p.to_lowercase().ends_with(".csv"))
        });
        let texts = self.read_many(&wanted)?;
        let doc = format::parse_problem(
            &texts.get(path).map(|b| text_of(b)).unwrap_or_default(),
            pid,
            slug,
        );
        let proposals = self.proposals(false)?;
        let summary = self.problem_summary(pid, path, &doc, &proposals);

        let works = self
            .files_under(&works_prefix)
            .filter_map(|f| self.work(f, &texts))
            .collect();

        Ok(crate::model::Problem {
            summary,
            body: doc
                .body
                .into_iter()
                .map(|(heading, text)| Section { heading, text })
                .collect(),
            works,
            events: self
                .problem_steps(pid)
                .into_iter()
                .map(|s| self.event(s))
                .collect(),
            notes: self.notes_under(&talk_prefix, &texts),
        })
    }

    fn work(&self, f: &TreeFile, texts: &HashMap<String, Vec<u8>>) -> Option<Work> {
        let text = texts.get(&f.path).map(|b| text_of(b));
        let fm = text.as_deref().map(|t| format::split_frontmatter(t).0);
        let WorkFile::Work(kind) = format::classify_work(&f.path, fm.as_ref()) else {
            return None;
        };
        let step = self.touched_step(&f.path);
        let mut title = fm.as_ref().and_then(|m| m.get("title")).map(str::to_string);
        let mut note = fm.as_ref().and_then(|m| m.get("note")).map(str::to_string);
        let mut size = f.size.unwrap_or(0);
        let mut blob = None;
        let mut body = None;
        match kind {
            WorkKind::Doc | WorkKind::Plan => {
                body = text
                    .as_deref()
                    .map(|t| format::split_frontmatter(t).1.to_string());
            }
            WorkKind::Table => {
                body = text.clone();
                let meta = format!("{}.meta.md", f.path);
                let meta = texts
                    .get(&meta)
                    .map(|b| format::split_frontmatter(&text_of(b)).0)
                    .unwrap_or_default();
                title = meta.get("title").map(str::to_string);
                note = meta.get("note").map(str::to_string);
            }
            WorkKind::Image | WorkKind::File => {
                if let Some(r) = text.as_deref().and_then(format::parse_blob_ref) {
                    size = r.size;
                    blob = Some(Blob {
                        cached: !r.purged && self.blobs_dir.join(&r.hash).is_file(),
                        hash: format!("sha256:{}", r.hash),
                        mime: r.mime,
                        purged: r.purged,
                    });
                }
            }
        }
        Some(Work {
            id: f.path.clone(),
            kind,
            title: title.unwrap_or_else(|| format::file_title(&f.path)),
            by: step.map(|s| self.by(s)).unwrap_or_default(),
            when: step.map(|s| when(s.time)).unwrap_or_default(),
            size,
            note,
            text: body,
            blob,
        })
    }

    // -- archive ------------------------------------------------------------

    fn problem_titles(&self) -> Result<HashMap<String, String>> {
        let files = self.problem_files();
        let paths: Vec<&str> = files.iter().map(|(_, _, p)| *p).collect();
        let texts = self.read_many(&paths)?;
        Ok(files
            .iter()
            .map(|(id, slug, p)| {
                let t = texts.get(*p).map(|b| text_of(b)).unwrap_or_default();
                (id.to_string(), format::parse_problem(&t, id, slug).title)
            })
            .collect())
    }

    /// The archive node for one file of `main`.
    fn leaf(
        &self,
        f: &TreeFile,
        titles: &HashMap<String, String>,
        texts: &HashMap<String, Vec<u8>>,
    ) -> ArchiveNode {
        let path = f.path.as_str();
        let text = texts.get(path).map(|b| text_of(b));
        let fm = text.as_deref().map(|t| format::split_frontmatter(t).0);
        let fm_title = fm.as_ref().and_then(|m| m.get("title")).map(str::to_string);
        let name = format::file_title(path);
        let (kind, label, problem) = if path == MEMBERS_PATH {
            (NodeKind::Members, None, None)
        } else if let Some((id, _)) = format::parse_problem_path(path) {
            (
                NodeKind::Problem,
                titles.get(id).cloned(),
                Some(id.to_string()),
            )
        } else if let Some((pid, _)) = format::parse_work_path(path) {
            let (kind, title) = match format::classify_work(path, fm.as_ref()) {
                WorkFile::Work(WorkKind::Table) => {
                    let meta = texts.get(&format!("{path}.meta.md")).map(|b| text_of(b));
                    let t = meta.and_then(|t| {
                        format::split_frontmatter(&t)
                            .0
                            .get("title")
                            .map(str::to_string)
                    });
                    (NodeKind::Table, t.unwrap_or(name))
                }
                WorkFile::Work(k) => (k.into(), fm_title.unwrap_or(name)),
                // Only the title of a table: shown by its file name.
                WorkFile::TableMeta => (NodeKind::File, file_name(path).to_string()),
            };
            (kind, Some(title), Some(pid.to_string()))
        } else if let Some(n) = format::parse_notice_path(path) {
            (
                NodeKind::Notice,
                Some(fm_title.unwrap_or_else(|| n.to_string())),
                None,
            )
        } else if let Some(pp) = format::parse_post_path(path) {
            // The member's name; the time is in the id (the stamp).
            let who = self
                .members
                .by_id(pp.member)
                .map_or(pp.member, |m| m.name.as_str());
            let problem = match pp.topic {
                TalkTopic::Problem(id) => Some(id.to_string()),
                TalkTopic::Board(_) => None,
            };
            (NodeKind::Note, Some(who.to_string()), problem)
        } else {
            let lower = path.to_lowercase();
            let kind = if lower.ends_with(".md") {
                NodeKind::Doc
            } else if lower.ends_with(".csv") {
                NodeKind::Table
            } else {
                NodeKind::File
            };
            (
                kind,
                Some(fm_title.unwrap_or_else(|| file_name(path).to_string())),
                None,
            )
        };
        ArchiveNode {
            id: path.to_string(),
            dir: None,
            label,
            kind,
            problem,
            size: f.size,
            children: None,
        }
    }

    /// Texts the archive needs for labels and kinds: markdown under `works/`
    /// and `board/`.
    fn label_texts(&self) -> Result<HashMap<String, Vec<u8>>> {
        let paths: Vec<&str> = self
            .tree
            .values()
            .filter(|f| {
                (f.path.starts_with("works/") || f.path.starts_with("board/"))
                    && f.path.ends_with(".md")
                    && f.size.unwrap_or(0) <= MAX_INLINE
            })
            .map(|f| f.path.as_str())
            .collect();
        self.read_many(&paths)
    }

    /// A directory's kind code (translated by the interface) or, for
    /// directories named after something people wrote, that title.
    fn dir_label(
        &self,
        dir: &str,
        titles: &HashMap<String, String>,
        texts: &HashMap<String, Vec<u8>>,
    ) -> (Option<&'static str>, Option<String>) {
        let parts: Vec<&str> = dir.split('/').collect();
        match parts.as_slice() {
            ["problems"] => (Some("problems"), None),
            ["works"] => (Some("works"), None),
            ["board"] | ["talk", "board"] => (Some("board"), None),
            ["talk"] => (Some("talk"), None),
            [".teahouse"] => (Some("meta"), None),
            ["works" | "talk", id] if titles.contains_key(*id) => (None, titles.get(*id).cloned()),
            ["talk", "board", notice] => {
                let title = texts.get(&format!("board/{notice}.md")).and_then(|b| {
                    format::split_frontmatter(&text_of(b))
                        .0
                        .get("title")
                        .map(str::to_string)
                });
                (None, Some(title.unwrap_or_else(|| notice.to_string())))
            }
            _ => (None, Some(parts.last().copied().unwrap_or(dir).to_string())),
        }
    }

    pub fn archive(&self, view: &str) -> Result<ArchiveNode> {
        let titles = self.problem_titles()?;
        let texts = self.label_texts()?;
        match view {
            "dirs" => Ok(self.archive_dirs(&titles, &texts)),
            "problems" => Ok(self.archive_problems(&titles, &texts)),
            _ => Err(Error::invalid(format!(
                "view must be 'dirs' or 'problems', not {view:?}"
            ))),
        }
    }

    fn archive_dirs(
        &self,
        titles: &HashMap<String, String>,
        texts: &HashMap<String, Vec<u8>>,
    ) -> ArchiveNode {
        // Build nested maps first, then nodes: directories before files.
        #[derive(Default)]
        struct Dir<'a> {
            dirs: BTreeMap<&'a str, Dir<'a>>,
            files: Vec<&'a TreeFile>,
        }
        let mut root = Dir::default();
        for f in self.tree.values() {
            // A table's title file belongs to the table (§1.2).
            if f.path.starts_with("works/")
                && format::classify_work(&f.path, None) == WorkFile::TableMeta
            {
                continue;
            }
            let mut d = &mut root;
            let mut parts: Vec<&str> = f.path.split('/').collect();
            parts.pop();
            // Files at the top (README.md) belong to the teahouse itself.
            if parts.is_empty() {
                parts.push(".teahouse");
            }
            for p in parts {
                d = d.dirs.entry(p).or_default();
            }
            d.files.push(f);
        }
        fn build(
            s: &Snapshot,
            path: String,
            code: Option<&'static str>,
            label: Option<String>,
            d: &Dir<'_>,
            titles: &HashMap<String, String>,
            texts: &HashMap<String, Vec<u8>>,
        ) -> ArchiveNode {
            let mut children: Vec<ArchiveNode> = d
                .dirs
                .iter()
                .map(|(name, sub)| {
                    let p = if path.is_empty() {
                        name.to_string()
                    } else {
                        format!("{path}/{name}")
                    };
                    let (code, label) = s.dir_label(&p, titles, texts);
                    build(s, p, code, label, sub, titles, texts)
                })
                .collect();
            children.extend(d.files.iter().map(|f| s.leaf(f, titles, texts)));
            let problem = path
                .strip_prefix("works/")
                .or_else(|| path.strip_prefix("talk/"))
                .filter(|id| titles.contains_key(*id))
                .map(str::to_string);
            ArchiveNode {
                id: path,
                dir: code,
                label,
                kind: NodeKind::Dir,
                problem,
                size: None,
                children: Some(children),
            }
        }
        build(
            self,
            String::new(),
            None,
            Some(self.members.name.clone()),
            &root,
            titles,
            texts,
        )
    }

    fn archive_problems(
        &self,
        titles: &HashMap<String, String>,
        texts: &HashMap<String, Vec<u8>>,
    ) -> ArchiveNode {
        let mut children = Vec::new();
        let mut placed: std::collections::HashSet<&str> = std::collections::HashSet::new();
        let posts_in = |prefix: &str| -> Vec<&TreeFile> {
            let mut v: Vec<&TreeFile> = self
                .files_under(prefix)
                .filter(|f| {
                    self.post_order.contains_key(&f.path)
                        && format::parse_post_path(&f.path).is_some()
                })
                .filter(|f| !f.path[prefix.len()..].contains('/'))
                .collect();
            v.sort_by_key(|f| self.post_order[&f.path]);
            v
        };
        for (id, _, path) in self.problem_files() {
            let f = &self.tree[path];
            placed.insert(path);
            let mut node = self.leaf(f, titles, texts);
            let mut kids = Vec::new();
            for w in self.files_under(&format!("works/{id}/")) {
                placed.insert(&w.path);
                // A table's title file belongs to the table, not beside it.
                if format::classify_work(&w.path, None) != WorkFile::TableMeta {
                    kids.push(self.leaf(w, titles, texts));
                }
            }
            for p in posts_in(&format!("talk/{id}/")) {
                placed.insert(&p.path);
                kids.push(self.leaf(p, titles, texts));
            }
            node.children = Some(kids);
            children.push(node);
        }
        let mut notices = Vec::new();
        for f in self.files_under("board/") {
            let Some(name) = format::parse_notice_path(&f.path) else {
                continue;
            };
            placed.insert(&f.path);
            let mut node = self.leaf(f, titles, texts);
            let replies: Vec<ArchiveNode> = posts_in(&format!("talk/board/{name}/"))
                .into_iter()
                .map(|p| {
                    placed.insert(&p.path);
                    self.leaf(p, titles, texts)
                })
                .collect();
            node.children = Some(replies);
            notices.push(node);
        }
        if !notices.is_empty() {
            children.push(ArchiveNode {
                id: "board".into(),
                dir: Some("board"),
                label: None,
                kind: NodeKind::Dir,
                problem: None,
                size: None,
                children: Some(notices),
            });
        }
        // The seating chart, README.md and anything not filed above.
        let rest: Vec<ArchiveNode> = self
            .tree
            .values()
            .filter(|f| !placed.contains(f.path.as_str()))
            .map(|f| self.leaf(f, titles, texts))
            .collect();
        if !rest.is_empty() {
            children.push(ArchiveNode {
                id: ".teahouse".into(),
                dir: Some("meta"),
                label: None,
                kind: NodeKind::Dir,
                problem: None,
                size: None,
                children: Some(rest),
            });
        }
        ArchiveNode {
            id: String::new(),
            dir: None,
            label: Some(self.members.name.clone()),
            kind: NodeKind::Dir,
            problem: None,
            size: None,
            children: Some(children),
        }
    }

    pub fn file(&self, path: &str) -> Result<FileView> {
        let Some(f) = self.tree.get(path) else {
            return Err(
                Error::not_found(format!("no file {path} / 没有这个文件")).with("path", path)
            );
        };
        let titles = self.problem_titles()?;
        let small = f.size.unwrap_or(0) <= MAX_INLINE;
        let meta = format!("{path}.meta.md");
        let texts = if small {
            self.read_many(&[path, &meta])?
        } else {
            HashMap::new()
        };
        let node = self.leaf(f, &titles, &texts);
        let text = texts
            .get(path)
            .filter(|_| small)
            .and_then(|b| String::from_utf8(b.clone()).ok())
            .filter(|t| !t.contains('\0'));
        let mut history = Vec::new();
        for s in &self.steps {
            if !s.changes.iter().any(|(_, p)| p == path) {
                continue;
            }
            let ev = self.event(s);
            let diff = if text.is_some() {
                self.git
                    .line_diff(s.parents.first().map(String::as_str), &s.hash, path)?
                    .map(|v| v.into_iter().map(diff_line).collect())
            } else {
                None
            };
            history.push(HistoryEntry {
                by: ev.by,
                when: ev.when,
                act: ev.act,
                problem: s.problem.first().cloned().unwrap_or_default(),
                title: ev.title,
                status: ev.status,
                diff,
            });
        }
        Ok(FileView {
            node,
            text,
            history,
        })
    }

    /// The local path of an attachment's content.
    pub fn blob(&self, path: &str) -> Result<String> {
        Ok(self.blob_file(path)?.0.to_string_lossy().into_owned())
    }

    /// The cached file of an attachment and its media type. Only ever a
    /// file in this engine's attachment cache, named by its hash.
    pub fn blob_file(&self, path: &str) -> Result<(PathBuf, String)> {
        let text = self
            .tree
            .get(path)
            .filter(|_| path.ends_with(".ref.md"))
            .map(|_| self.read_many(&[path]))
            .transpose()?
            .and_then(|mut m| m.remove(path))
            .ok_or_else(|| {
                Error::not_found(format!("{path} is not an attachment / 不是附件"))
                    .with("path", path)
            })?;
        let r = format::parse_blob_ref(&text_of(&text)).ok_or_else(|| {
            Error::invalid(format!(
                "{path}: malformed attachment reference / 附件引用格式不对"
            ))
        })?;
        if r.purged {
            return Err(
                Error::not_found("the host removed this attachment / 老板已删除这个附件")
                    .with("path", path)
                    .with("reason", "purged"),
            );
        }
        let local = self.blobs_dir.join(&r.hash);
        if local.is_file() {
            return Ok((local, r.mime));
        }
        Err(Error::new(
            Code::Offline,
            "attachment not downloaded yet / 附件还没有下载",
        )
        .with("path", path))
    }
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn diff_line((c, text): (char, String)) -> DiffLine {
    DiffLine {
        kind: match c {
            '+' => LineKind::Add,
            '-' => LineKind::Del,
            _ => LineKind::Same,
        },
        text,
    }
}
