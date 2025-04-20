//! [Start] definition.

use ::clap::{ArgMatches, Command};

mod sealed {
    //! [Sealed] trait definition.

    /// Sealed traits.
    pub trait Sealed {}

    impl<T> Sealed for crate::startable::Startable<T> {}
}

/// Trait for types that may be created and ran, intended for use as a dynamic trait using
/// [startable][crate::startable]
pub trait Start: sealed::Sealed {
    /// Start as a subcommand of an application, without setup of log.
    ///
    /// # Errors
    /// As a result of [Run::run][crate::Run::run].
    fn start_as_subcommand(&self, matches: &ArgMatches) -> crate::Result;

    /// Attach [LogConfig][::log_level_cli::LogConfig], parse and call [run][crate::Run::run].
    ///
    /// # Errors
    /// If panic handler cannot be installed or the [Run::run][crate::Run::run] implementation needs to.
    fn start_as_application(&self, modules: &[&str]) -> crate::Result;

    /// Command without [LogConfig][::log_level_cli::LogConfig] attached.
    fn command_as_subcommand(&self) -> Command;

    /// Get command with [LogConfig][::log_level_cli::LogConfig] attached.
    fn command_as_application(&self) -> Command;
}
