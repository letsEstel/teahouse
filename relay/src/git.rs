//! Thin, explicit wrappers around the `git` binary.
//!
//! The relay never re-implements git. Every question about objects, history
//! or trailers is answered by git itself, so the relay agrees with git about
//! what a commit *is*. Output is parsed in `-z` (NUL-separated) form wherever
//! git offers it, so unusual file names cannot confuse the parser.

use crate::bail;
use crate::error::{Error, Result};
use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// Environment variables that would redirect git away from the repository we
/// mean. Cleared for admin operations; kept inside hooks (where git sets them
/// on purpose, e.g. for the object quarantine).
const REDIRECTING_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_QUARANTINE_PATH",
    "GIT_NAMESPACE",
    "GIT_CEILING_DIRECTORIES",
    "GIT_COMMON_DIR",
];

/// One entry of a tree, as `git ls-tree -l` reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeEntry {
    /// `100644`, `100755`, `120000` (symlink), `160000` (submodule), `040000`.
    pub mode: String,
    /// `blob`, `tree` or `commit`.
    pub kind: String,
    /// Size in bytes; `None` for trees and submodules.
    pub size: Option<u64>,
}

impl TreeEntry {
    /// A regular file (executable or not): not a symlink, directory or submodule.
    pub fn is_regular_file(&self) -> bool {
        self.kind == "blob" && (self.mode == "100644" || self.mode == "100755")
    }
}

#[derive(Debug, Clone)]
pub struct Git {
    /// `Some(dir)`: operate on that repository with a clean environment.
    /// `None`: inherit the environment git gave us (hook mode).
    dir: Option<PathBuf>,
    extra_env: Vec<(String, String)>,
}

pub fn is_zero_oid(oid: &str) -> bool {
    !oid.is_empty() && oid.bytes().all(|b| b == b'0')
}

fn is_oid(s: &str) -> bool {
    (s.len() == 40 || s.len() == 64)
        && s.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

impl Git {
    pub fn at(dir: impl Into<PathBuf>) -> Git {
        Git {
            dir: Some(dir.into()),
            extra_env: Vec::new(),
        }
    }

    /// For use inside a git hook: repository and quarantine come from git.
    pub fn inherited() -> Git {
        Git {
            dir: None,
            extra_env: Vec::new(),
        }
    }

    pub fn with_env(mut self, k: &str, v: &str) -> Git {
        self.extra_env.push((k.into(), v.into()));
        self
    }

    fn command(&self) -> Command {
        let mut c = Command::new("git");
        if let Some(dir) = &self.dir {
            for v in REDIRECTING_ENV {
                c.env_remove(v);
            }
            c.arg("--git-dir").arg(dir);
        }
        // Never let user/system config change how we read history.
        c.env("GIT_CONFIG_NOSYSTEM", "1");
        c.env("GIT_TERMINAL_PROMPT", "0");
        c.env("LC_ALL", "C");
        for (k, v) in &self.extra_env {
            c.env(k, v);
        }
        c
    }

    fn run_raw<I, S>(&self, args: I, stdin: Option<&[u8]>) -> Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args: Vec<_> = args.into_iter().map(|a| a.as_ref().to_owned()).collect();
        let mut cmd = self.command();
        cmd.args(&args);
        cmd.stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = cmd
            .spawn()
            .map_err(|e| Error(format!("cannot run git (is it installed?): {e}")))?;
        if let Some(input) = stdin {
            let mut si = child.stdin.take().expect("piped");
            si.write_all(input)?;
        }
        Ok(child.wait_with_output()?)
    }

    fn describe(args: &[&str]) -> String {
        format!("git {}", args.join(" "))
    }

    /// Runs git and returns stdout; any non-zero exit is an error.
    pub fn run(&self, args: &[&str]) -> Result<Vec<u8>> {
        self.run_input(args, None)
    }

    pub fn run_input(&self, args: &[&str], stdin: Option<&[u8]>) -> Result<Vec<u8>> {
        let out = self.run_raw(args, stdin)?;
        if !out.status.success() {
            bail!(
                "{} failed ({}): {}",
                Self::describe(args),
                out.status,
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(out.stdout)
    }

    pub fn run_str(&self, args: &[&str]) -> Result<String> {
        let out = self.run(args)?;
        String::from_utf8(out)
            .map(|s| s.trim_end_matches('\n').to_string())
            .map_err(|_| {
                Error(format!(
                    "{} returned non-UTF-8 output",
                    Self::describe(args)
                ))
            })
    }

    /// Resolves a revision to an object id, or `None` if it does not exist.
    pub fn resolve(&self, rev: &str) -> Result<Option<String>> {
        let spec = format!("{rev}^{{commit}}");
        let out = self.run_raw(
            [
                "rev-parse",
                "--verify",
                "--quiet",
                "--end-of-options",
                spec.as_str(),
            ],
            None,
        )?;
        match out.status.code() {
            Some(0) => {
                let oid = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !is_oid(&oid) {
                    bail!("git rev-parse returned {oid:?}");
                }
                Ok(Some(oid))
            }
            Some(1) => Ok(None),
            _ => bail!(
                "git rev-parse {rev}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        }
    }

    /// Reads `path` at `commit`, or `None` if the path does not exist there.
    pub fn read_file(&self, commit: &str, path: &str) -> Result<Option<Vec<u8>>> {
        let spec = format!("{commit}:{path}");
        let out = self.run_raw(["cat-file", "-e", spec.as_str()], None)?;
        if !out.status.success() {
            return Ok(None);
        }
        let kind = self.run_str(&["cat-file", "-t", &spec])?;
        if kind != "blob" {
            bail!("{path} at {commit} is a {kind}, expected a file");
        }
        Ok(Some(self.run(&["cat-file", "blob", &spec])?))
    }

    pub fn is_ancestor(&self, ancestor: &str, descendant: &str) -> Result<bool> {
        let out = self.run_raw(["merge-base", "--is-ancestor", ancestor, descendant], None)?;
        match out.status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => bail!(
                "git merge-base: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        }
    }

    /// `git rev-list <args>`; returns object ids.
    pub fn rev_list(&self, args: &[&str]) -> Result<Vec<String>> {
        let mut full = vec!["rev-list"];
        full.extend_from_slice(args);
        let out = self.run_str(&full)?;
        Ok(out
            .lines()
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect())
    }

    pub fn parents(&self, commit: &str) -> Result<Vec<String>> {
        let line = self.run_str(&["rev-list", "--parents", "-n", "1", commit])?;
        Ok(line.split(' ').skip(1).map(str::to_string).collect())
    }

    /// Paths a commit introduces. For a merge, only paths that differ from
    /// *every* parent (`-c`) — i.e. changes the merge itself made, which no
    /// parent can vouch for. Renames are reported as delete + add, so a file
    /// cannot be moved out of a protected directory unnoticed.
    pub fn changed_paths(&self, commit: &str) -> Result<Vec<String>> {
        let parents = self.parents(commit)?;
        let out = if parents.len() >= 2 {
            self.run(&[
                "diff-tree",
                "--no-commit-id",
                "-r",
                "-c",
                "--no-renames",
                "--name-only",
                "-z",
                commit,
            ])?
        } else {
            self.run(&[
                "diff-tree",
                "--no-commit-id",
                "-r",
                "--root",
                "--no-renames",
                "--name-only",
                "-z",
                commit,
            ])?
        };
        split_z(out)
    }

    /// `(status, path)` for a non-merge commit: `A`dded, `M`odified,
    /// `D`eleted, `T`ype-changed. Renames appear as `D` + `A`.
    pub fn changes(&self, commit: &str) -> Result<Vec<(char, String)>> {
        let out = self.run(&[
            "diff-tree",
            "--no-commit-id",
            "-r",
            "--root",
            "--no-renames",
            "--name-status",
            "-z",
            commit,
        ])?;
        let fields = split_z(out)?;
        let mut v = Vec::new();
        for pair in fields.chunks(2) {
            match pair {
                [status, path] => v.push((status.chars().next().unwrap_or('?'), path.clone())),
                _ => bail!("unexpected diff-tree output for {commit}"),
            }
        }
        Ok(v)
    }

    /// Values of the `key:` trailer, parsed by git's own trailer parser.
    pub fn trailer_values(&self, commit: &str, key: &str) -> Result<Vec<String>> {
        let fmt = format!("--format=%(trailers:key={key},valueonly,unfold,separator=%x00)");
        let out = self.run(&["log", "-1", "--no-show-signature", &fmt, commit])?;
        let s = String::from_utf8(out).map_err(|_| Error("trailer is not UTF-8".into()))?;
        let s = s.trim_end_matches('\n');
        if s.is_empty() {
            return Ok(Vec::new());
        }
        Ok(s.split('\0').map(|v| v.trim().to_string()).collect())
    }

    /// File names directly inside `dir/` at `commit` (no recursion).
    /// The tree entry at `path` in `commit` without reading its content;
    /// `None` if there is nothing at that path.
    pub fn tree_entry(&self, commit: &str, path: &str) -> Result<Option<TreeEntry>> {
        let out = self.run(&["ls-tree", "-l", "-z", "--full-tree", commit, "--", path])?;
        for rec in split_z(out)? {
            // <mode> SP <type> SP <oid> SP+ <size or -> TAB <path>
            let Some((meta, p)) = rec.split_once('\t') else {
                bail!("unexpected ls-tree output for {commit}:{path}");
            };
            if p != path {
                continue;
            }
            let mut it = meta.split_whitespace();
            let (Some(mode), Some(kind), Some(_oid), Some(size)) =
                (it.next(), it.next(), it.next(), it.next())
            else {
                bail!("unexpected ls-tree output for {commit}:{path}");
            };
            return Ok(Some(TreeEntry {
                mode: mode.to_string(),
                kind: kind.to_string(),
                size: size.parse().ok(),
            }));
        }
        Ok(None)
    }
    pub fn list_dir(&self, commit: &str, dir: &str) -> Result<Vec<String>> {
        let spec = format!("{commit}:{dir}");
        let exists = self.run_raw(["cat-file", "-e", spec.as_str()], None)?;
        if !exists.status.success() {
            return Ok(Vec::new());
        }
        let out = self.run(&["ls-tree", "--name-only", "-z", &spec])?;
        split_z(out)
    }

    pub fn hash_blob(&self, data: &[u8]) -> Result<String> {
        let oid = String::from_utf8(self.run_input(&["hash-object", "-w", "--stdin"], Some(data))?)
            .map_err(|_| Error("bad oid".into()))?;
        Ok(oid.trim().to_string())
    }
}

fn split_z(out: Vec<u8>) -> Result<Vec<String>> {
    out.split(|&b| b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| String::from_utf8(p.to_vec()).map_err(|_| Error("path is not valid UTF-8".into())))
        .collect()
}

/// Identity used for commits the relay itself writes (opening, succession).
pub struct Signature<'a> {
    pub name: &'a str,
    pub email: &'a str,
}

/// Builds a commit from `files` layered over `base_tree_of` (or empty), using
/// a private index file so no working tree is ever involved.
pub fn commit_files(
    git: &Git,
    scratch: &Path,
    base: Option<&str>,
    files: &[(&str, &[u8])],
    message: &str,
    who: &Signature<'_>,
) -> Result<String> {
    let index = scratch.join(format!("teahouse-index-{}", std::process::id()));
    let _ = std::fs::remove_file(&index);
    let idx = index.to_string_lossy().to_string();
    let g = git.clone().with_env("GIT_INDEX_FILE", &idx);
    let result = (|| {
        match base {
            Some(b) => g.run(&["read-tree", b])?,
            None => g.run(&["read-tree", "--empty"])?,
        };
        for (path, data) in files {
            let blob = g.hash_blob(data)?;
            let info = format!("100644,{blob},{path}");
            g.run(&["update-index", "--add", "--cacheinfo", &info])?;
        }
        let tree = g.run_str(&["write-tree"])?;
        let mut args = vec!["commit-tree", tree.as_str()];
        if let Some(b) = base {
            args.push("-p");
            args.push(b);
        }
        let g2 = g
            .clone()
            .with_env("GIT_AUTHOR_NAME", who.name)
            .with_env("GIT_AUTHOR_EMAIL", who.email)
            .with_env("GIT_COMMITTER_NAME", who.name)
            .with_env("GIT_COMMITTER_EMAIL", who.email);
        let out = g2.run_input(
            &[&args[..], &["-F", "-"]].concat(),
            Some(message.as_bytes()),
        )?;
        Ok(String::from_utf8_lossy(&out).trim().to_string())
    })();
    let _ = std::fs::remove_file(&index);
    result
}
