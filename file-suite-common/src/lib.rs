//! Common utilities for crates in workspace.

use ::clap::{Args, Command, CommandFactory, FromArgMatches};
use ::log_level_cli::LogConfig;

/// Common cli required and provided functions.
pub trait Cli {
    /// Run this cli.
    ///
    /// # Errors
    /// If the implementation whishes to.
    fn run(self) -> ::color_eyre::Result<()>;

    /// Attach [LogConfig], parse and call [run][Cli::run].
    ///
    /// # Errors
    /// If panic handler cannot be installed or the implementation whishes to.
    fn start<I, S>(modules: I) -> ::color_eyre::Result<()>
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

        cli.run()
    }

    /// Get command with [LogConfig] attached.
    fn standalone_command() -> Command
    where
        Self: CommandFactory,
    {
        LogConfig::augment_args(Self::command())
    }
}
