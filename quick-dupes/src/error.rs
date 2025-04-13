use ::std::{fmt::Display, path::PathBuf};

use ::thiserror::Error;

#[derive(Debug, Error)]
#[error("{path:?} could not be canonicalized, {err}")]
pub struct CanonicalizationError {
    pub path: PathBuf,
    #[source]
    pub err: ::std::io::Error,
}

pub fn log_if_err<F: FnOnce() -> Result<T, E>, T, E: Display>(l: ::log::Level, f: F) -> Option<T> {
    f().map_err(|err| ::log::log!(l, "\n{err}")).ok()
}
