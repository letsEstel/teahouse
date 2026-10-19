//! The engine's errors: `ApiError` of CLIENT-API.md §5.

use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Code {
    NotFound,
    NotAMember,
    Forbidden,
    Invalid,
    Conflict,
    Offline,
    RelayRefused,
    TooLarge,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Error {
    pub code: Code,
    /// What the interface needs to word the error itself, e.g. `{path}`.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, String>,
    /// A fallback sentence, for codes the interface does not know.
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(code: Code, message: impl Into<String>) -> Error {
        Error {
            code,
            params: BTreeMap::new(),
            message: message.into(),
            reasons: Vec::new(),
        }
    }

    /// Adds a parameter for the interface's own wording.
    pub fn with(mut self, key: &str, value: impl Into<String>) -> Error {
        self.params.insert(key.to_string(), value.into());
        self
    }

    pub fn internal(message: impl Into<String>) -> Error {
        Error::new(Code::Internal, message)
    }

    pub fn not_found(message: impl Into<String>) -> Error {
        Error::new(Code::NotFound, message)
    }

    pub fn invalid(message: impl Into<String>) -> Error {
        Error::new(Code::Invalid, message)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::internal(e.to_string())
    }
}
