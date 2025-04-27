use ::std::{fmt::Display, path::PathBuf};

use ::thiserror::Error;

/// Error used when a path cannot be canonicalized.
#[derive(Debug, Error)]
#[error("{path:?} could not be canonicalized, {err}")]
pub struct CanonicalizationError {
    /// Path that could not be canonicalized.
    pub path: PathBuf,
    /// IO error that occurred.
    #[source]
    pub err: ::std::io::Error,
}

pub fn log_if_err<F: FnOnce() -> Result<T, E>, T, E: Display>(l: ::log::Level, f: F) -> Option<T> {
    f().map_err(|err| ::log::log!(l, "\n{err}")).ok()
}
