//! Common utilities for crates in workspace.

use ::clap::{CommandFactory, FromArgMatches};

/// Common cli required and provided functions.
pub trait Cli {
    /// Run cli.
    ///
    /// # Errors
    /// If the implementation whishes to.
    fn run(self) -> ::color_eyre::Result<()>;

    /// Attach log config, parse and run cli.
    ///
    /// # Errors
    /// If panic handler cannot be installed or the implementation whishes to.
    fn start() -> ::color_eyre::Result<()>
    where
        Self: CommandFactory + FromArgMatches,
    {
        let command = Self::command();
        let matches = command.get_matches();


        let cli = Self::from_arg_matches(&matches).unwrap_or_else(|err| err.exit());

        Ok(())
    }
}
