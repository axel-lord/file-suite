//! [Startable] implementation.

use ::std::marker::PhantomData;

use ::clap::{Args, Command, CommandFactory, FromArgMatches};
use ::color_eyre::Report;
use ::log_level_cli::LogConfig;

use crate::{Run, Start};

/// Get a dynamic [Start] reference from a [Run] implementation.
pub fn startable<T>() -> &'static dyn Start
where
    T: Run + CommandFactory + FromArgMatches + 'static,
    T::Err: Send + Sync,
    Report: From<T::Err>,
{
    let startable = Startable { _p: PhantomData };
    let boxed = Box::<Startable<T>>::new(startable);
    Box::<dyn Start>::leak(boxed)
}

/// Implementation for [Start] trait, allowing for the creation of dyn [Start] values.
pub(crate) struct Startable<T> {
    /// Allow for T to exist.
    _p: PhantomData<fn() -> T>,
}

impl<T> Start for Startable<T>
where
    T: Run + CommandFactory + FromArgMatches,
    T::Err: Send + Sync,
    Report: From<T::Err>,
{
    fn start_as_application(&self, modules: &[&str]) -> crate::Result {
        ::color_eyre::install()?;

        let command = self.command_as_application();
        let matches = command.get_matches();

        let log_config = LogConfig::from_arg_matches(&matches).unwrap_or_else(|err| err.exit());

        log_config.init(modules);

        let cli = T::from_arg_matches(&matches).unwrap_or_else(|err| err.exit());

        cli.run().map_err(Report::from)
    }

    fn command_as_application(&self) -> Command {
        LogConfig::augment_args(T::command())
    }

    fn command_as_subcommand(&self) -> Command {
        T::command()
    }

    fn start_as_subcommand(&self, matches: &clap::ArgMatches) -> crate::Result {
        let cli = T::from_arg_matches(matches).unwrap_or_else(|err| err.exit());
        cli.run().map_err(Report::from)
    }
}
