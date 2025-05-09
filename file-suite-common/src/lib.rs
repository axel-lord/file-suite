//! Common utilities for crates in workspace.

use ::std::process::ExitCode;

use ::clap::{CommandFactory, FromArgMatches};
use ::color_eyre::Report;

use crate::startable::Startable;
pub use crate::{run::Run, start::Start, startable::startable};

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

/// Invoke [Start::start] for T as if it using [startable].
///
/// # Errors
/// If panic handler cannot be installed or the [Run::run] implementation needs to.
pub fn start<T>(modules: &[&str]) -> Result
where
    T: Run + CommandFactory + FromArgMatches + 'static,
    T::Err: Send + Sync,
    Report: From<T::Err>,
{
    Startable::<T>::new().start_as_application(modules)
}
