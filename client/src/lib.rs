//! # teahouse-client
//!
//! The client engine of docs/CLIENT-API.md: every teahouse is a full local
//! clone, and the interface reaches it only through the commands of §4,
//! never through git. Git is libgit2 inside the process; no git program is
//! ever run, so the same engine builds for Android.
//!
//! Layout of the engine's root directory:
//!
//! ```text
//! <root>/houses/<house-id>/   the teahouse's git repository (bare)
//! <root>/blobs/<sha256>       local copies of attachments (§1.6)
//! <root>/device, device.pub   this device's key (made on first use)
//! <root>/known_hosts          relay host keys, pinned on first use
//! <root>/drafts/<house>/      what people wrote when the relay refused it
//! ```

pub mod commit;
pub mod error;
pub mod format;
pub mod git;
pub mod house;
pub mod identity;
pub mod model;
pub mod serve;
pub mod sync;
pub mod transport;
pub mod write;

use error::{Code, Error, Result};
use house::Snapshot;
use identity::Identity;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use teahouse_core::PublicKey;
use write::{ChangeArg, SectionArg, WorkKindArg, Writer};

pub struct Client {
    root: PathBuf,
}

/// House ids name directories; keep them to what the relay allows.
fn valid_house_id(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && !id.starts_with(['.', '-'])
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}

/// A git repository: a clone with a work tree, or a bare one.
fn is_repository(dir: &Path) -> bool {
    dir.join(".git").exists() || dir.join("HEAD").is_file()
}

impl Client {
    pub fn new(root: impl Into<PathBuf>) -> Client {
        Client { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn houses_dir(&self) -> PathBuf {
        self.root.join("houses")
    }

    pub(crate) fn known_hosts(&self) -> PathBuf {
        self.root.join("known_hosts")
    }

    fn device_key(&self) -> Option<PublicKey> {
        let line = std::fs::read_to_string(self.root.join("device.pub")).ok()?;
        PublicKey::parse(line.trim()).ok()
    }

    /// This device's key, made on first use.
    pub fn identity(&self) -> Result<Identity> {
        Identity::load_or_create(&self.root)
    }

    pub fn snapshot(&self, house: &str) -> Result<Snapshot> {
        let dir = self.houses_dir().join(house);
        if !valid_house_id(house) || !is_repository(&dir) {
            return Err(Error::not_found(format!(
                "no teahouse {house:?} on this device / 本机没有这间茶馆"
            ))
            .with("house", house));
        }
        Snapshot::load(
            house,
            &dir,
            &self.root.join("blobs"),
            self.device_key().as_ref(),
        )
    }

    /// Every teahouse cloned on this device, by id.
    pub fn house_ids(&self) -> Result<Vec<String>> {
        let mut ids: Vec<String> = match std::fs::read_dir(self.houses_dir()) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .filter(|e| is_repository(&e.path()))
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|n| valid_house_id(n))
                .collect(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e.into()),
        };
        ids.sort();
        Ok(ids)
    }

    pub fn houses(&self) -> Result<Vec<model::HouseSummary>> {
        self.house_ids()?
            .iter()
            .map(|id| self.snapshot(id)?.summary())
            .collect()
    }

    /// Syncs one teahouse and returns where it stands. Being offline or
    /// refused is a state here, not an error.
    pub fn sync_house(&self, house: &str) -> Result<model::SyncState> {
        let snap = self.snapshot(house)?;
        let identity = self.identity()?;
        let link = sync::Link {
            repo: snap.git.repo(),
            identity: &identity,
            known_hosts: &self.known_hosts(),
        };
        if let Err(e) = link.sync() {
            if e.code == Code::Internal {
                return Err(e);
            }
            // Anything else is recorded and shows in the state.
        }
        self.snapshot(house)?.summary_sync()
    }

    /// Clones a teahouse from a relay with this device's key.
    pub fn join(&self, relay: &str, house: &str) -> Result<model::HouseSummary> {
        if !valid_house_id(house) {
            return Err(
                Error::invalid(format!("not a teahouse id: {house:?}")).with("house", house)
            );
        }
        let dir = self.houses_dir().join(house);
        if dir.exists() {
            return Err(Error::invalid(format!(
                "teahouse {house} is already on this device / 本机已经有这间茶馆"
            ))
            .with("house", house));
        }
        let url = transport::house_url(relay, house)?;
        let identity = self.identity()?;
        std::fs::create_dir_all(self.houses_dir())?;
        let repo = git2::Repository::init_bare(&dir)?;
        repo.remote("origin", &url)?;
        let link = sync::Link {
            repo: &repo,
            identity: &identity,
            known_hosts: &self.known_hosts(),
        };
        if let Err(e) = link.sync() {
            drop(repo);
            let _ = std::fs::remove_dir_all(&dir);
            return Err(e);
        }
        self.snapshot(house)?.summary()
    }

    /// Keeps what someone wrote when the relay refused it; returns the path
    /// relative to the engine root.
    pub(crate) fn save_draft(
        &self,
        house: &str,
        command: &str,
        args: &Value,
        error: &Error,
    ) -> Result<String> {
        let rel = format!("drafts/{house}/{}-{command}.json", sync::now());
        let path = self.root.join(&rel);
        std::fs::create_dir_all(path.parent().expect("has a parent"))?;
        let draft = json!({ "command": command, "args": args, "error": error });
        std::fs::write(&path, serde_json::to_vec_pretty(&draft).expect("json"))?;
        Ok(rel)
    }

    /// Runs one command of CLIENT-API.md §4 and returns its envelope:
    /// `{ok: true, data}` or `{ok: false, error}`.
    pub fn call(&self, command: &str, args: Value) -> Value {
        match self.dispatch(command, args) {
            Ok(data) => json!({ "ok": true, "data": data }),
            Err(error) => json!({ "ok": false, "error": error }),
        }
    }

    fn dispatch(&self, command: &str, args: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct HouseArg {
            house: String,
        }
        #[derive(Deserialize)]
        struct MaybeHouse {
            house: Option<String>,
        }
        #[derive(Deserialize)]
        struct ProblemArg {
            house: String,
            problem: String,
        }
        #[derive(Deserialize)]
        struct ArchiveArg {
            house: String,
            view: String,
        }
        #[derive(Deserialize)]
        struct IdArg {
            house: String,
            id: String,
        }
        #[derive(Deserialize)]
        struct JoinArg {
            relay: String,
            house: String,
        }
        #[derive(Deserialize)]
        struct OpenProblemArg {
            house: String,
            title: String,
            #[serde(default)]
            body: Vec<SectionArg>,
        }
        #[derive(Deserialize)]
        struct SetStatusArg {
            house: String,
            problem: String,
            status: model::Status,
            owner: Option<String>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct AddWorkArg {
            house: String,
            problem: String,
            kind: WorkKindArg,
            title: String,
            text: Option<String>,
            from_path: Option<String>,
            note: Option<String>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct UpdateWorkArg {
            house: String,
            id: String,
            text: Option<String>,
            from_path: Option<String>,
        }
        #[derive(Deserialize)]
        struct NoteArg {
            house: String,
            problem: Option<String>,
            notice: Option<String>,
            text: String,
        }
        #[derive(Deserialize)]
        struct NoticeArg {
            house: String,
            title: String,
            text: String,
        }
        #[derive(Deserialize)]
        struct ProposeArg {
            house: String,
            problem: String,
            title: String,
            reason: Option<String>,
            changes: Vec<ChangeArg>,
        }
        #[derive(Deserialize)]
        struct ProposalArg {
            house: String,
            proposal: String,
            reason: Option<String>,
        }
        #[derive(Deserialize)]
        struct RoleArg {
            id: String,
            role: String,
        }
        #[derive(Deserialize)]
        struct SeatArg {
            house: String,
            remove: Option<String>,
            role: Option<RoleArg>,
        }
        fn parse<T: for<'de> Deserialize<'de>>(command: &str, v: &Value) -> Result<T> {
            let v = if v.is_null() { json!({}) } else { v.clone() };
            serde_json::from_value(v)
                .map_err(|e| Error::invalid(format!("{command}: bad arguments: {e}")))
        }
        fn to_value<T: serde::Serialize>(t: T) -> Result<Value> {
            serde_json::to_value(t).map_err(|e| Error::internal(e.to_string()))
        }
        let a = &args;
        match command {
            "houses" => to_value(self.houses()?),
            "house" => {
                let a: HouseArg = parse(command, a)?;
                to_value(self.snapshot(&a.house)?.house()?)
            }
            "problem" => {
                let a: ProblemArg = parse(command, a)?;
                to_value(self.snapshot(&a.house)?.problem(&a.problem)?)
            }
            "archive" => {
                let a: ArchiveArg = parse(command, a)?;
                to_value(self.snapshot(&a.house)?.archive(&a.view)?)
            }
            "file" => {
                let a: IdArg = parse(command, a)?;
                to_value(self.snapshot(&a.house)?.file(&a.id)?)
            }
            "blob" => {
                let a: IdArg = parse(command, a)?;
                to_value(self.snapshot(&a.house)?.blob(&a.id)?)
            }
            "sync" => {
                let a: MaybeHouse = parse(command, a)?;
                match a.house {
                    Some(h) => to_value(self.sync_house(&h)?),
                    None => {
                        let mut all = serde_json::Map::new();
                        for h in self.house_ids()? {
                            all.insert(h.clone(), to_value(self.sync_house(&h)?)?);
                        }
                        Ok(Value::Object(all))
                    }
                }
            }
            "join" => {
                let a: JoinArg = parse(command, a)?;
                to_value(self.join(&a.relay, &a.house)?)
            }
            "openProblem" => {
                let p: OpenProblemArg = parse(command, a)?;
                to_value(Writer::open(self, &p.house)?.open_problem(&p.title, &p.body, a)?)
            }
            "setStatus" => {
                let p: SetStatusArg = parse(command, a)?;
                to_value(Writer::open(self, &p.house)?.set_status(
                    &p.problem,
                    p.status,
                    p.owner.as_deref(),
                    a,
                )?)
            }
            "addWork" => {
                let p: AddWorkArg = parse(command, a)?;
                to_value(Writer::open(self, &p.house)?.add_work(
                    &p.problem,
                    p.kind,
                    &p.title,
                    p.text.as_deref(),
                    p.from_path.as_deref(),
                    p.note.as_deref(),
                    a,
                )?)
            }
            "updateWork" => {
                let p: UpdateWorkArg = parse(command, a)?;
                to_value(Writer::open(self, &p.house)?.update_work(
                    &p.id,
                    p.text.as_deref(),
                    p.from_path.as_deref(),
                    a,
                )?)
            }
            "note" => {
                let p: NoteArg = parse(command, a)?;
                to_value(Writer::open(self, &p.house)?.note(
                    p.problem.as_deref(),
                    p.notice.as_deref(),
                    &p.text,
                    a,
                )?)
            }
            "removeNote" => {
                let p: IdArg = parse(command, a)?;
                to_value(Writer::open(self, &p.house)?.remove_note(&p.id, a)?)
            }
            "postNotice" => {
                let p: NoticeArg = parse(command, a)?;
                to_value(Writer::open(self, &p.house)?.post_notice(&p.title, &p.text, a)?)
            }
            "propose" => {
                let p: ProposeArg = parse(command, a)?;
                to_value(Writer::open(self, &p.house)?.propose(
                    &p.problem,
                    &p.title,
                    p.reason.as_deref(),
                    &p.changes,
                    a,
                )?)
            }
            "withdraw" => {
                let p: ProposalArg = parse(command, a)?;
                to_value(Writer::open(self, &p.house)?.withdraw(&p.proposal, a)?)
            }
            "adopt" => {
                let p: ProposalArg = parse(command, a)?;
                to_value(Writer::open(self, &p.house)?.adopt(&p.proposal, a)?)
            }
            "decline" => {
                let p: ProposalArg = parse(command, a)?;
                to_value(Writer::open(self, &p.house)?.decline(
                    &p.proposal,
                    p.reason.as_deref(),
                    a,
                )?)
            }
            "seat" => {
                let p: SeatArg = parse(command, a)?;
                let role = p.role.as_ref().map(|r| (r.id.as_str(), r.role.as_str()));
                to_value(Writer::open(self, &p.house)?.seat(p.remove.as_deref(), role, a)?)
            }
            "export" | "invite" | "redeem" | "purgeBlob" => Err(Error::invalid(format!(
                "{command} is not implemented yet / 这条命令还没有实现"
            ))
            .with("command", command)),
            _ => {
                Err(Error::invalid(format!("unknown command {command:?}")).with("command", command))
            }
        }
    }
}
