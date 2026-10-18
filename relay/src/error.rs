use std::fmt;

/// The relay's single error type: a human-readable message.
/// Errors are reported, never swallowed; the process exits non-zero.
#[derive(Debug)]
pub struct Error(pub String);

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error(format!("I/O error: {e}"))
    }
}

impl From<teahouse_core::members::MembersError> for Error {
    fn from(e: teahouse_core::members::MembersError) -> Self {
        Error(e.to_string())
    }
}

impl From<teahouse_core::key::KeyError> for Error {
    fn from(e: teahouse_core::key::KeyError) -> Self {
        Error(format!("key: {e}"))
    }
}

#[macro_export]
macro_rules! bail {
    ($($t:tt)*) => { return Err($crate::error::Error(format!($($t)*))) };
}

pub trait Context<T> {
    fn context(self, what: impl FnOnce() -> String) -> Result<T>;
}

impl<T, E: fmt::Display> Context<T> for std::result::Result<T, E> {
    fn context(self, what: impl FnOnce() -> String) -> Result<T> {
        self.map_err(|e| Error(format!("{}: {e}", what())))
    }
}
