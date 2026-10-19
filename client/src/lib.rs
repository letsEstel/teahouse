//! # teahouse-client
//!
//! The client engine of docs/CLIENT-API.md: every teahouse is a full local
//! clone, and the interface reaches it only through the commands of §4,
//! never through git. This crate has the read side (step 1 of §7).
//!
//! Layout of the engine's root directory:
//!
//! ```text
//! <root>/houses/<house-id>/   a normal git clone of the teahouse
//! <root>/blobs/<sha256>       local copies of attachments (§1.6)
//! <root>/device.pub           this device's public key (OpenSSH line)
//! ```

pub mod error;
pub mod format;
pub mod git;
pub mod house;
pub mod model;

use error::{Error, Result};
use house::Snapshot;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use teahouse_core::PublicKey;

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

    fn device_key(&self) -> Option<PublicKey> {
        let line = std::fs::read_to_string(self.root.join("device.pub")).ok()?;
        PublicKey::parse(line.trim()).ok()
    }

    pub fn snapshot(&self, house: &str) -> Result<Snapshot> {
        let dir = self.houses_dir().join(house);
        if !valid_house_id(house) || !dir.join(".git").exists() {
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
    pub fn houses(&self) -> Result<Vec<model::HouseSummary>> {
        let mut ids: Vec<String> = match std::fs::read_dir(self.houses_dir()) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .filter(|e| e.path().join(".git").exists())
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|n| valid_house_id(n))
                .collect(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e.into()),
        };
        ids.sort();
        ids.iter().map(|id| self.snapshot(id)?.summary()).collect()
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
        fn parse<T: for<'de> Deserialize<'de>>(command: &str, v: Value) -> Result<T> {
            let v = if v.is_null() { json!({}) } else { v };
            serde_json::from_value(v)
                .map_err(|e| Error::invalid(format!("{command}: bad arguments: {e}")))
        }
        fn to_value<T: serde::Serialize>(t: T) -> Result<Value> {
            serde_json::to_value(t).map_err(|e| Error::internal(e.to_string()))
        }
        match command {
            "houses" => to_value(self.houses()?),
            "house" => {
                let a: HouseArg = parse(command, args)?;
                to_value(self.snapshot(&a.house)?.house()?)
            }
            "problem" => {
                let a: ProblemArg = parse(command, args)?;
                to_value(self.snapshot(&a.house)?.problem(&a.problem)?)
            }
            "archive" => {
                let a: ArchiveArg = parse(command, args)?;
                to_value(self.snapshot(&a.house)?.archive(&a.view)?)
            }
            "file" => {
                let a: IdArg = parse(command, args)?;
                to_value(self.snapshot(&a.house)?.file(&a.id)?)
            }
            "blob" => {
                let a: IdArg = parse(command, args)?;
                to_value(self.snapshot(&a.house)?.blob(&a.id)?)
            }
            "export" | "sync" | "join" | "openProblem" | "setStatus" | "addWork" | "updateWork"
            | "note" | "postNotice" | "propose" | "withdraw" | "adopt" | "decline" | "seat"
            | "invite" | "redeem" | "purgeBlob" => Err(Error::invalid(format!(
                "{command} is not implemented yet / 这条命令还没有实现"
            ))),
            _ => Err(Error::invalid(format!("unknown command {command:?}"))),
        }
    }
}
