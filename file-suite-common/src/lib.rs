//! Common utilities for crates in workspace.

use ::clap::{Args, Command, CommandFactory, FromArgMatches};
use ::log_level_cli::LogConfig;

/// Re-export of [::color_eyre::Result] for use in crates that do not use [color_eyre].
pub type Result<T = ()> = ::color_eyre::Result<T>;

/// Common cli required and provided functions.
pub trait Cli {
    /// Error used by cli.
    type Err: Into<::color_eyre::Report> + Send + Sync;

    /// Run this cli.
    ///
    /// # Errors
    /// If the implementation whishes to.
    fn run(self) -> ::core::result::Result<(), Self::Err>;

    /// Attach [LogConfig], parse and call [run][Cli::run].
    ///
    /// # Errors
    /// If panic handler cannot be installed or the implementation whishes to.
    fn start<I, S>(modules: I) -> self::Result
    where
        Self: CommandFactory + FromArgMatches,
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        ::color_eyre::install()?;

        let command = Self::standalone_command();
        let matches = command.get_matches();

        let log_config = LogConfig::from_arg_matches(&matches).unwrap_or_else(|err| err.exit());

        log_config.init(modules);

        let cli = Self::from_arg_matches(&matches).unwrap_or_else(|err| err.exit());

        cli.run().map_err(|err| err.into())
    }

    /// Get command with [LogConfig] attached.
    fn standalone_command() -> Command
    where
        Self: CommandFactory,
    {
        LogConfig::augment_args(Self::command())
    }
}
