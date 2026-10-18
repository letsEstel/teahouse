//! The relay's on-disk layout.
//!
//! ```text
//! <root>/
//!   relay.conf          key = value settings (written by `init`)
//!   houses/<id>.git     one bare repository per teahouse
//!   hooks/              pre-receive / post-receive, shared by every house
//!   log/audit.log       append-only JSON lines, one per decision
//!   tmp/                scratch space for index files
//! ```

use crate::bail;
use crate::error::{Context, Result};
use crate::git::Git;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use teahouse_core::members::{Members, MEMBERS_PATH};
use teahouse_core::rules::MAIN_REF;

pub const DEFAULT_ROOT: &str = "/var/lib/teahouse";

#[derive(Debug, Clone)]
pub struct Root {
    pub dir: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Absolute path of the relay binary, used in hooks and authorized_keys.
    pub binary: String,
    /// If set, `post-receive` rewrites this authorized_keys file when seats change.
    pub authorized_keys: Option<String>,
}

/// Characters allowed in paths we embed into shell scripts and the
/// `command="..."` of authorized_keys. Anything else is refused outright.
pub fn safe_path(p: &str) -> bool {
    p.starts_with('/')
        && p.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-+@".contains(&b))
        && !p.contains("/../")
        && !p.ends_with("/..")
}

/// House ids name directories and appear in clone URLs.
pub fn valid_house_id(id: &str) -> bool {
    let b = id.as_bytes();
    (1..=63).contains(&b.len())
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter()
            .all(|&c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

impl Root {
    pub fn new(dir: impl Into<PathBuf>) -> Root {
        Root { dir: dir.into() }
    }

    pub fn conf_path(&self) -> PathBuf {
        self.dir.join("relay.conf")
    }
    pub fn houses_dir(&self) -> PathBuf {
        self.dir.join("houses")
    }
    pub fn hooks_dir(&self) -> PathBuf {
        self.dir.join("hooks")
    }
    pub fn log_path(&self) -> PathBuf {
        self.dir.join("log").join("audit.log")
    }
    pub fn tmp_dir(&self) -> PathBuf {
        self.dir.join("tmp")
    }

    pub fn house_path(&self, id: &str) -> PathBuf {
        self.houses_dir().join(format!("{id}.git"))
    }

    pub fn dir_str(&self) -> Result<String> {
        let Some(s) = self.dir.to_str().map(str::to_string) else {
            bail!("relay root path must be UTF-8");
        };
        if !safe_path(&s) {
            bail!("relay root {s:?} must be an absolute path of [A-Za-z0-9/._-+@] characters");
        }
        Ok(s)
    }

    pub fn load_config(&self) -> Result<Config> {
        let path = self.conf_path();
        let text = fs::read_to_string(&path).context(|| {
            format!(
                "cannot read {} (did you run `teahouse-relay init`?)",
                path.display()
            )
        })?;
        let mut map = BTreeMap::new();
        for (i, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                bail!("{}:{}: expected key = value", path.display(), i + 1);
            };
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
        let mut cfg = Config::default();
        for (k, v) in map {
            match k.as_str() {
                "binary" => cfg.binary = v,
                "authorized_keys" => cfg.authorized_keys = Some(v),
                other => bail!("{}: unknown setting {other:?}", path.display()),
            }
        }
        if !safe_path(&cfg.binary) {
            bail!("{}: binary must be a safe absolute path", path.display());
        }
        if let Some(a) = &cfg.authorized_keys {
            if !safe_path(a) {
                bail!(
                    "{}: authorized_keys must be a safe absolute path",
                    path.display()
                );
            }
        }
        Ok(cfg)
    }

    pub fn house_ids(&self) -> Result<Vec<String>> {
        let mut ids = Vec::new();
        let dir = self.houses_dir();
        if !dir.exists() {
            return Ok(ids);
        }
        for entry in fs::read_dir(&dir)? {
            let name = entry?.file_name();
            let Some(name) = name.to_str() else { continue };
            if let Some(id) = name.strip_suffix(".git") {
                if valid_house_id(id) {
                    ids.push(id.to_string());
                }
            }
        }
        ids.sort();
        Ok(ids)
    }

    pub fn house(&self, id: &str) -> Result<House> {
        if !valid_house_id(id) {
            bail!("invalid teahouse id {id:?}");
        }
        let path = self.house_path(id);
        if !path.join("HEAD").exists() {
            bail!("no teahouse {id:?}");
        }
        Ok(House {
            id: id.to_string(),
            git: Git::at(&path),
            path,
        })
    }
}

#[allow(dead_code)]
pub struct House {
    pub id: String,
    pub path: PathBuf,
    pub git: Git,
}

impl House {
    /// The seating chart that is in force: members.toml on the current main.
    pub fn members(&self) -> Result<Members> {
        members_at(&self.git, MAIN_REF)
    }
}

pub fn members_at(git: &Git, rev: &str) -> Result<Members> {
    let Some(oid) = git.resolve(rev)? else {
        bail!("{rev} does not exist");
    };
    let Some(bytes) = git.read_file(&oid, MEMBERS_PATH)? else {
        bail!("{MEMBERS_PATH} is missing at {rev}");
    };
    let text = match String::from_utf8(bytes) {
        Ok(t) => t,
        Err(_) => bail!("{MEMBERS_PATH} is not UTF-8"),
    };
    Ok(Members::parse(&text)?)
}

pub fn write_atomic(path: &Path, data: &[u8], mode: u32) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let dir = path.parent().unwrap_or(Path::new("."));
    let tmp = dir.join(format!(
        ".{}.tmp-{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("file"),
        std::process::id()
    ));
    {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(mode)
            .open(&tmp)
            .context(|| format!("cannot write {}", tmp.display()))?;
        f.write_all(data)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path).context(|| format!("cannot replace {}", path.display()))?;
    if let Ok(d) = fs::File::open(dir) {
        let _ = d.sync_all();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_and_paths() {
        assert!(valid_house_id("wutong-3"));
        assert!(!valid_house_id("../x"));
        assert!(!valid_house_id("Wutong"));
        assert!(!valid_house_id(""));
        assert!(!valid_house_id("-x"));
        assert!(safe_path("/var/lib/teahouse"));
        assert!(!safe_path("relative"));
        assert!(!safe_path("/a b"));
        assert!(!safe_path("/a\"b"));
        assert!(!safe_path("/a/../b"));
        assert!(!safe_path("/a;rm"));
    }
}
