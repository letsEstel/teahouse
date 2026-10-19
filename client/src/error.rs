//! The engine's errors: `ApiError` of CLIENT-API.md §5.

use serde::Serialize;
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
    /// One sentence for people, Chinese and English like the relay's.
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(code: Code, message: impl Into<String>) -> Error {
        Error {
            code,
            message: message.into(),
            reasons: Vec::new(),
        }
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
