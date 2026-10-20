//! # teahouse-core
//!
//! The data model of a teahouse (茶馆), with no dependencies and no I/O:
//! members and roles, member keys, and the authorisation rules the relay
//! enforces. The same crate will back the desktop client, so the client can
//! warn before a push the relay would reject.

pub mod frontmatter;
pub mod key;
pub mod members;
pub mod rules;
pub mod sha256;
pub mod toml;

pub use key::PublicKey;
pub use members::{Device, Kind, Member, Members, Role, MAX_DEVICES, MAX_MEMBERS, MEMBERS_PATH};
pub use rules::{ProblemRef, RefKind};
