//! This device's key (CLIENT-API.md §2): one ed25519 key, made on first use.
//! It signs every commit (SSHSIG, the format of git's `gpg.format=ssh`) and
//! authenticates to the relay.
//!
//! The private key lives in `<root>/device` (OpenSSH format, mode 0600).
//! Moving it into the system keychain is a packaging step (§7 step 4); the
//! rest of the engine only uses [`Identity`].

use crate::error::{Error, Result};
use ssh_key::{Algorithm, HashAlg, LineEnding, PrivateKey};
use std::path::Path;
use teahouse_core::PublicKey;

/// The namespace git signs commits in.
const GIT_NAMESPACE: &str = "git";

pub struct Identity {
    key: PrivateKey,
    /// `<root>/device`, for ssh libraries that cannot take a key from memory
    /// (libssh2 on Windows).
    path: std::path::PathBuf,
}

fn io(e: impl std::fmt::Display) -> Error {
    Error::internal(format!("device key: {e}"))
}

impl Identity {
    /// Loads `<root>/device`, making a new key on first use.
    pub fn load_or_create(root: &Path) -> Result<Identity> {
        let path = root.join("device");
        if path.exists() {
            let text = std::fs::read_to_string(&path)?;
            let key = PrivateKey::from_openssh(text).map_err(io)?;
            if key.algorithm() != Algorithm::Ed25519 {
                return Err(io("the device key must be ed25519"));
            }
            return Ok(Identity { key, path });
        }
        std::fs::create_dir_all(root)?;
        let key =
            PrivateKey::random(&mut ssh_key::rand_core::OsRng, Algorithm::Ed25519).map_err(io)?;
        let pem = key.to_openssh(LineEnding::LF).map_err(io)?;
        write_private(&path, pem.as_bytes())?;
        let id = Identity {
            key,
            path: path.clone(),
        };
        std::fs::write(root.join("device.pub"), format!("{}\n", id.public_line()))?;
        Ok(id)
    }

    /// `ssh-ed25519 AAAA… teahouse-device`
    pub fn public_line(&self) -> String {
        let mut public = self.key.public_key().clone();
        public.set_comment("teahouse-device");
        public.to_openssh().expect("ed25519 public keys encode")
    }

    pub fn public(&self) -> PublicKey {
        PublicKey::parse(&self.public_line()).expect("an ed25519 key parses")
    }

    /// The private key's file (OpenSSH format).
    pub fn private_path(&self) -> &Path {
        &self.path
    }

    /// The private key in OpenSSH format, for libssh2.
    pub fn private_openssh(&self) -> Result<String> {
        Ok(self.key.to_openssh(LineEnding::LF).map_err(io)?.to_string())
    }

    /// An armored SSHSIG over `data`, as git writes into a commit's
    /// `gpgsig` header.
    pub fn sign(&self, data: &[u8]) -> Result<String> {
        let sig = self
            .key
            .sign(GIT_NAMESPACE, HashAlg::Sha512, data)
            .map_err(io)?;
        sig.to_pem(LineEnding::LF).map_err(io)
    }
}

#[cfg(unix)]
fn write_private(path: &Path, data: &[u8]) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    f.write_all(data)?;
    Ok(())
}

#[cfg(not(unix))]
fn write_private(path: &Path, data: &[u8]) -> Result<()> {
    std::fs::write(path, data)?;
    Ok(())
}
