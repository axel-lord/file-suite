//! Log level config.

use ::clap::{Args, ValueEnum};
use ::env_logger::Target;
use ::log::LevelFilter;
use ::tap::Pipe;

/// Configure logging.
#[derive(Args)]
#[command(next_help_heading = "Logging Options")]
pub struct Log {
    /// Where to write log.
    #[arg(long, visible_alias = "lf", default_value_t, hide_short_help = true)]
    log_file: patharg::OutputArg,

    /// Highest log level.
    #[arg(long, visible_alias = "ll", value_enum, default_value_t, hide_short_help = true)]
    log_level: Filter,
}

impl Log {
    /// Set up log
    pub fn setup(self) -> color_eyre::Result<()> {
        let file = match self.log_file {
            ::patharg::OutputArg::Stdout => Target::Stderr,
            ::patharg::OutputArg::Path(path) => std::fs::File::options()
                .create(true)
                .append(true)
                .open(path)?
                .pipe(|file| Target::Pipe(Box::new(file))),
        };

        env_logger::builder()
            .filter_module("re_tile", self.log_level.into())
            .target(file)
            .init();

        Ok(())
    }
}

/// Level to filter logs at.
#[repr(usize)]
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Filter {
    /// No logging.
    Off = 0,
    /// Log only errors.
    Error = 1,
    /// Log warnings or worse.
    Warn = 2,
    /// Log info or worse.
    Info = 3,
    /// Log debug or worse.
    Debug = 4,
    /// Log everyting.
    Trace = 5,
}

impl Default for Filter {
    fn default() -> Self {
        const {
            if cfg!(debug_assertions) {
                Filter::Trace
            } else {
                Filter::Info
            }
        }
    }
}

impl From<Filter> for LevelFilter {
    fn from(value: Filter) -> Self {
        match value {
            Filter::Off => LevelFilter::Off,
            Filter::Error => LevelFilter::Error,
            Filter::Warn => LevelFilter::Warn,
            Filter::Info => LevelFilter::Info,
            Filter::Debug => LevelFilter::Debug,
            Filter::Trace => LevelFilter::Trace,
        }
    }
}

