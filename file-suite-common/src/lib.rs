//! Common utilities for crates in workspace.

use ::std::process::ExitCode;

use ::clap::{CommandFactory, FromArgMatches};
use ::color_eyre::Report;

pub use crate::{run::Run, start::Start, startable::Startable};

mod run;
mod start;
mod startable;

/// Re-export of [::color_eyre::Result] for use in crates that do not use [color_eyre].
pub type Result<T = ()> = ::std::result::Result<T, Report>;

/// Error wrapping an exit code.
#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq)]
#[error("exit code {:?}", .0)]
pub struct ExitCodeError(pub ExitCode);

impl From<ExitCode> for ExitCodeError {
    fn from(value: ExitCode) -> Self {
        Self(value)
    }
}

impl From<u8> for ExitCodeError {
    fn from(value: u8) -> Self {
        Self(value.into())
    }
}

/// Invoke [Start::start] for T as if it using [Startable].
///
/// # Errors
/// If panic handler cannot be installed or the [Run::run] implementation needs to.
pub fn start<T>(modules: &[&str]) -> Result
where
    T: Run + CommandFactory + FromArgMatches,
    T::Err: Send + Sync,
    Report: From<T::Err>,
{
    Startable::<T>::new().start(modules)
}
