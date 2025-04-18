//! [Startable] implementation.

use ::std::marker::PhantomData;

use ::clap::{Args, Command, CommandFactory, FromArgMatches};
use ::color_eyre::Report;
use ::log_level_cli::LogConfig;

use crate::{Run, Start};

/// Implementation for [Start] trait, allowing for the creation of `dyn [Start]` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Startable<T> {
    /// Allow for T to exist.
    _p: PhantomData<fn() -> T>,
}

impl<T> Startable<T>
where
    T: Run + CommandFactory + FromArgMatches,
    T::Err: Send + Sync,
    Report: From<T::Err>,
{
    /// Create a new instance.
    pub const fn new() -> Self {
        Self { _p: PhantomData }
    }
}

impl<T> Default for Startable<T>
where
    T: Run + CommandFactory + FromArgMatches,
    T::Err: Send + Sync,
    Report: From<T::Err>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Start for Startable<T>
where
    T: Run + CommandFactory + FromArgMatches,
    T::Err: Send + Sync,
    Report: From<T::Err>,
{
    fn start(&self, modules: &[&str]) -> crate::Result {
        ::color_eyre::install()?;

        let command = self.standalone_command();
        let matches = command.get_matches();

        let log_config = LogConfig::from_arg_matches(&matches).unwrap_or_else(|err| err.exit());

        log_config.init(modules);

        let cli = T::from_arg_matches(&matches).unwrap_or_else(|err| err.exit());

        cli.run().map_err(Report::from)
    }

    fn standalone_command(&self) -> Command {
        LogConfig::augment_args(T::command())
    }
}
