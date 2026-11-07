//! Which lock an engine command waits for in the shell (see lib.rs).
//! Kept apart from Tauri so `rustc --test src/turn.rs` checks it anywhere.

#[derive(Debug, PartialEq)]
pub enum Turn {
    /// Only reads the local copy: runs at once, even during a sync.
    Free,
    /// `blob`: may download into `blobs/` (downloads share a temporary name).
    Blob,
    /// Everything else writes or syncs: one at a time.
    Engine,
}

pub fn turn(command: &str) -> Turn {
    match command {
        "houses" | "house" | "problem" | "archive" | "file" | "devices" => Turn::Free,
        "blob" => Turn::Blob,
        _ => Turn::Engine,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Page loads must not wait for a sync (0024, 0034); anything that writes must.
    #[test]
    fn reads_skip_the_lock_writes_take_it() {
        for c in ["houses", "house", "problem", "archive", "file", "devices"] {
            assert_eq!(turn(c), Turn::Free, "{c}");
        }
        assert_eq!(turn("blob"), Turn::Blob);
        for c in [
            "sync", "note", "addWork", "updateWork", "propose", "adopt", "decline", "drafts", "discardDraft",
            "export", "redeem", "join", "pair", "unpair", "purgeBlob", "seat", "rename", "invite",
        ] {
            assert_eq!(turn(c), Turn::Engine, "{c}");
        }
    }
}
