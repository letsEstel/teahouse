//! `saveFile {from, name}`: copy an attachment out of the app's attachment
//! store into a folder the person can open (Download/Teahouse on Android,
//! the download folder on a computer), so it opens in another app (0036).
//! Kept apart from Tauri so `rustc --test src/save.rs` checks it anywhere.

use std::path::{Path, PathBuf};

/// Copies `from` (which must lie inside `blobs`) into `dir` as `name`, and
/// returns the new path. Only the last component of `name` is used, and an
/// existing file is never overwritten: "a.pdf" becomes "a (2).pdf".
pub fn save_copy(blobs: &Path, from: &str, dir: &Path, name: &str) -> Result<String, String> {
    let from = Path::new(from).canonicalize().map_err(|e| format!("{from}: {e}"))?;
    let blobs = blobs.canonicalize().map_err(|e| e.to_string())?;
    if !from.starts_with(&blobs) || !from.is_file() {
        return Err("only attachments can be saved this way".into());
    }
    let base = Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.trim().is_empty() && *n != "." && *n != "..")
        .unwrap_or("file");
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let dest = free_name(dir, base);
    std::fs::copy(&from, &dest).map_err(|e| e.to_string())?;
    dest.into_os_string().into_string().map_err(|_| "path is not UTF-8".to_string())
}

fn free_name(dir: &Path, base: &str) -> PathBuf {
    let (stem, ext) = match base.rfind('.') {
        Some(i) if i > 0 => (&base[..i], &base[i..]),
        _ => (base, ""),
    };
    let mut path = dir.join(base);
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{stem} ({n}){ext}"));
        n += 1;
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("th-save-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("blobs")).unwrap();
        d
    }

    #[test]
    fn copies_without_overwriting() {
        let d = scratch("copy");
        let blob = d.join("blobs/abc");
        std::fs::write(&blob, b"%PDF").unwrap();
        let out = d.join("out");
        let a = save_copy(&d.join("blobs"), blob.to_str().unwrap(), &out, "Weber.pdf").unwrap();
        let b = save_copy(&d.join("blobs"), blob.to_str().unwrap(), &out, "Weber.pdf").unwrap();
        assert!(a.ends_with("Weber.pdf"), "{a}");
        assert!(b.ends_with("Weber (2).pdf"), "{b}");
        assert_eq!(std::fs::read(&b).unwrap(), b"%PDF");
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn refuses_files_outside_the_store_and_strips_folders_from_the_name() {
        let d = scratch("outside");
        let secret = d.join("device.key");
        std::fs::write(&secret, b"key").unwrap();
        assert!(save_copy(&d.join("blobs"), secret.to_str().unwrap(), &d.join("out"), "k").is_err());
        let sneaky = d.join("blobs/../device.key");
        assert!(save_copy(&d.join("blobs"), sneaky.to_str().unwrap(), &d.join("out"), "k").is_err());
        let blob = d.join("blobs/abc");
        std::fs::write(&blob, b"x").unwrap();
        let p = save_copy(&d.join("blobs"), blob.to_str().unwrap(), &d.join("out"), "../../evil.txt").unwrap();
        assert_eq!(Path::new(&p).parent().unwrap(), d.join("out").canonicalize().unwrap_or(d.join("out")));
        let _ = std::fs::remove_dir_all(d);
    }
}
