//! The attachment store (CLIENT-API.md §1.6): files named by their SHA-256
//! under `<root>/blobs/<house>/`, reached over the same ssh channel as git.
//! Seated members upload and download; only the host deletes (purge).

use crate::audit::{self, Entry};
use crate::bail;
use crate::error::Result;
use crate::root::Root;
use std::fs;
use std::io::{Read, Write};
use teahouse_core::frontmatter::MAX_BLOB_BYTES;
use teahouse_core::sha256::{valid_hex, Sha256};

/// Default total size of one teahouse's attachments: 2 GiB.
pub const DEFAULT_QUOTA: u64 = 2 * 1024 * 1024 * 1024;

fn check_hash(hash: &str) -> Result<()> {
    if !valid_hex(hash) {
        bail!("{hash:?} is not a sha256 hex digest");
    }
    Ok(())
}

/// Bytes stored for a teahouse.
pub fn usage(root: &Root, house: &str) -> u64 {
    let Ok(rd) = fs::read_dir(root.blobs_dir(house)) else {
        return 0;
    };
    rd.filter_map(|e| e.ok())
        .filter(|e| valid_hex(&e.file_name().to_string_lossy()))
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

pub fn has(root: &Root, house: &str, hash: &str) -> Result<bool> {
    check_hash(hash)?;
    Ok(root.blobs_dir(house).join(hash).is_file())
}

/// Reads exactly `size` bytes from `input`, checks they hash to `hash`,
/// and stores them. Uploading something already stored is a no-op.
pub fn put(
    root: &Root,
    house: &str,
    member: &str,
    hash: &str,
    size: u64,
    input: &mut dyn Read,
    quota: u64,
) -> Result<()> {
    check_hash(hash)?;
    if size > MAX_BLOB_BYTES {
        bail!("attachments are limited to 100 MiB / 附件最大 100 MiB");
    }
    let dir = root.blobs_dir(house);
    fs::create_dir_all(&dir)?;
    let dest = dir.join(hash);
    let exists = dest.is_file();
    if !exists && usage(root, house) + size > quota {
        bail!(
            "this teahouse's attachments would exceed {} MiB / 这间茶馆的附件总量超出上限",
            quota / (1024 * 1024)
        );
    }
    let tmp = dir.join(format!(".upload-{}-{hash}", std::process::id()));
    let result = (|| {
        let mut out = if exists {
            None
        } else {
            Some(fs::File::create(&tmp)?)
        };
        let mut h = Sha256::new();
        let mut left = size;
        let mut buf = vec![0u8; 64 * 1024];
        while left > 0 {
            let want = buf.len().min(left as usize);
            let n = input.read(&mut buf[..want])?;
            if n == 0 {
                bail!("upload ended after {} of {size} bytes", size - left);
            }
            h.update(&buf[..n]);
            if let Some(f) = out.as_mut() {
                f.write_all(&buf[..n])?;
            }
            left -= n as u64;
        }
        let got = h.finish_hex();
        if got != hash {
            bail!("the upload hashes to {got}, not {hash} / 附件内容和哈希不符");
        }
        if let Some(f) = out {
            f.sync_all()?;
            fs::rename(&tmp, &dest)?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result?;
    audit::log(
        root,
        Entry::new("blob.put")
            .house(house)
            .member(member)
            .field("blob", hash.to_string())
            .field("size", size.to_string()),
    );
    Ok(())
}

pub fn get(root: &Root, house: &str, hash: &str, output: &mut dyn Write) -> Result<()> {
    check_hash(hash)?;
    let mut f = match fs::File::open(root.blobs_dir(house).join(hash)) {
        Ok(f) => f,
        Err(_) => bail!("no attachment sha256:{hash} here / 没有这个附件"),
    };
    std::io::copy(&mut f, output)?;
    output.flush()?;
    Ok(())
}

/// Deletes an attachment's content for good (the host's `purgeBlob`).
pub fn purge(root: &Root, house: &str, member: &str, hash: &str) -> Result<()> {
    check_hash(hash)?;
    let path = root.blobs_dir(house).join(hash);
    let existed = path.is_file();
    if existed {
        fs::remove_file(&path)?;
    }
    audit::log(
        root,
        Entry::new("blob.purged")
            .house(house)
            .member(member)
            .field("blob", hash.to_string())
            .field("existed", existed.to_string()),
    );
    Ok(())
}
