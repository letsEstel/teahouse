//! OpenSSH ed25519 public keys — the identity of a teahouse member.
//!
//! Only `ssh-ed25519` is accepted. A key is validated structurally (wire
//! format, algorithm name, 32-byte point) and stored in canonical form so two
//! spellings of the same key can never count as two people.

use std::fmt;

const ALG: &str = "ssh-ed25519";

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PublicKey {
    /// Canonical base64 of the SSH wire blob.
    blob_b64: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyError(pub String);

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for KeyError {}

impl PublicKey {
    /// Parses `ssh-ed25519 <base64> [comment]`. The comment is discarded.
    pub fn parse(line: &str) -> Result<Self, KeyError> {
        let mut parts = line.split_ascii_whitespace();
        let alg = parts.next().ok_or_else(|| KeyError("empty key".into()))?;
        if alg != ALG {
            return Err(KeyError(format!(
                "unsupported key type {alg:?}; teahouse accepts only ssh-ed25519"
            )));
        }
        let b64 = parts
            .next()
            .ok_or_else(|| KeyError("key is missing its base64 body".into()))?;
        Self::from_blob_b64(b64)
    }

    /// Parses the bare base64 blob (as passed to the forced command).
    pub fn from_blob_b64(b64: &str) -> Result<Self, KeyError> {
        let blob =
            base64_decode(b64).ok_or_else(|| KeyError("key body is not valid base64".into()))?;
        let mut r = Reader { buf: &blob };
        let alg = r.string().ok_or_else(|| KeyError("truncated key".into()))?;
        if alg != ALG.as_bytes() {
            return Err(KeyError("key body does not match ssh-ed25519".into()));
        }
        let point = r.string().ok_or_else(|| KeyError("truncated key".into()))?;
        if point.len() != 32 {
            return Err(KeyError("ed25519 key must be 32 bytes".into()));
        }
        if !r.buf.is_empty() {
            return Err(KeyError("trailing bytes in key".into()));
        }
        Ok(PublicKey {
            blob_b64: base64_encode(&blob),
        })
    }

    /// Builds a key from a raw 32-byte ed25519 point.
    pub fn from_point(point: &[u8; 32]) -> Self {
        let mut blob = Vec::with_capacity(51);
        blob.extend_from_slice(&(ALG.len() as u32).to_be_bytes());
        blob.extend_from_slice(ALG.as_bytes());
        blob.extend_from_slice(&32u32.to_be_bytes());
        blob.extend_from_slice(point);
        PublicKey {
            blob_b64: base64_encode(&blob),
        }
    }

    pub fn blob_b64(&self) -> &str {
        &self.blob_b64
    }

    /// `ssh-ed25519 <base64>` (no comment).
    pub fn openssh(&self) -> String {
        format!("{ALG} {}", self.blob_b64)
    }

    /// A short, human-friendly tag for logs (not a security fingerprint).
    pub fn short(&self) -> String {
        let b = &self.blob_b64;
        let tail = &b[b.len().saturating_sub(12)..];
        format!("ed25519:…{}", tail.trim_end_matches('='))
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({})", self.short())
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.openssh())
    }
}

struct Reader<'a> {
    buf: &'a [u8],
}

impl<'a> Reader<'a> {
    fn string(&mut self) -> Option<&'a [u8]> {
        if self.buf.len() < 4 {
            return None;
        }
        let len = u32::from_be_bytes(self.buf[..4].try_into().ok()?) as usize;
        let rest = &self.buf[4..];
        if rest.len() < len {
            return None;
        }
        let (s, tail) = rest.split_at(len);
        self.buf = tail;
        Some(s)
    }
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            B64[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// Strict standard base64 decoding (padding required, no whitespace).
pub fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.as_bytes();
    if s.is_empty() || s.len() % 4 != 0 {
        return None;
    }
    let val = |c: u8| -> Option<u32> {
        Some(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        } as u32)
    };
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let last = s.len() / 4 - 1;
    for (i, q) in s.chunks(4).enumerate() {
        let pad = q.iter().rev().take_while(|&&c| c == b'=').count();
        if pad > 2 || (pad > 0 && i != last) || q[..4 - pad].contains(&b'=') {
            return None;
        }
        let mut n = 0u32;
        for &c in &q[..4 - pad] {
            n = (n << 6) | val(c)?;
        }
        n <<= 6 * pad as u32;
        let bytes = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
        // Reject non-canonical encodings (non-zero bits in padding positions).
        if pad == 1 && bytes[2] != 0 || pad == 2 && (bytes[1] != 0 || bytes[2] != 0) {
            return None;
        }
        out.extend_from_slice(&bytes[..3 - pad]);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // A real key produced by `ssh-keygen -t ed25519`.
    const REAL: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl user@host";

    #[test]
    fn parses_real_key() {
        let k = PublicKey::parse(REAL).unwrap();
        assert_eq!(
            k.openssh(),
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl"
        );
        assert_eq!(PublicKey::from_blob_b64(k.blob_b64()).unwrap(), k);
    }

    #[test]
    fn rejects_bad_keys() {
        for bad in [
            "",
            "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQ",
            "ssh-ed25519",
            "ssh-ed25519 not-base64!",
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GK",
            // valid base64, wrong inner algorithm
            "ssh-ed25519 AAAAB3NzaC1yc2EAAAADAQAB",
        ] {
            assert!(PublicKey::parse(bad).is_err(), "should reject {bad:?}");
        }
    }

    #[test]
    fn base64_vectors() {
        for (raw, enc) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64_encode(raw.as_bytes()), enc);
            if !enc.is_empty() {
                assert_eq!(base64_decode(enc).unwrap(), raw.as_bytes());
            }
        }
        assert!(base64_decode("Zg=").is_none());
        assert!(base64_decode("Zh==").is_none(), "non-canonical");
        assert!(base64_decode("Zg==Zg==").is_none());
    }

    #[test]
    fn from_point_roundtrip() {
        let k = PublicKey::from_point(&[7u8; 32]);
        assert_eq!(PublicKey::parse(&k.openssh()).unwrap(), k);
    }
}
