use ::std::{fs, num::NonZero, path::PathBuf, thread};

use ::clap::{Args, Parser, ValueEnum, builder::ArgPredicate::Equals};
use ::color_eyre::{Section, eyre::eyre};
use ::derive_more::IsVariant;
use ::rayon::ThreadPoolBuilder;

use crate::error::CanonicalizationError;

/// Get amount of threads that shouls be used.
fn default_thread_count() -> NonZero<usize> {
    if let Ok(t) = thread::available_parallelism() {
        return t;
    }
    if let Some(t) = NonZero::new(1) {
        return t;
    }
    unreachable!()
}

/// How should files containing newline characters be treated.
#[derive(Clone, Copy, Debug, ValueEnum, IsVariant)]
pub enum NewLineBehaviour {
    /// Skip files containing newlines.
    #[value(alias = "s")]
    Skip,
    /// Include files containing newlines.
    #[value(alias = "i")]
    Include,
}

/// Yes/No response.
#[derive(Clone, Copy, Debug, ValueEnum, IsVariant)]
pub enum Response {
    /// Yes
    #[value(alias = "y")]
    Yes,
    /// No
    #[value(alias = "n")]
    No,
}

impl From<Response> for bool {
    fn from(value: Response) -> Self {
        matches!(value, Response::Yes)
    }
}

impl From<bool> for Response {
    fn from(value: bool) -> Self {
        if value { Response::Yes } else { Response::No }
    }
}

/// Filter configuration.
#[derive(Debug, Args)]
#[command(next_help_heading = "Filters")]
pub struct Filter {
    /// Filter files using provided regex.
    #[arg(long, visible_alias = "re", value_parser = ::regex::bytes::Regex::new)]
    pub regex: Option<::regex::bytes::Regex>,

    /// Minimum depth to search.
    ///
    /// At 0 the search starts with the given paths, at 1 it's directory contents.
    #[arg(long, default_value_t = usize::MIN)]
    pub min_depth: usize,

    /// Maximum depth to search.
    ///
    /// Only search entries up to and including the given depth, at 0 the search only includes the
    /// given paths.
    ///
    /// To disable recursion specify 1.
    #[arg(long, short = 'd', default_value_t = usize::MAX)]
    pub max_depth: usize,

    /// How to handle filepaths with newlines.
    ///
    /// Default value depends on --print-zero, and should be appropriate for most situations.
    #[arg(
        long,
        visible_alias = "nl",
        value_enum,
        default_value_if("print_zero", Equals("true".into()), "include"),
        default_value = "skip",
    )]
    pub newlines: NewLineBehaviour,

    /// Use name when searching.
    #[arg(
        long = "match-name",
        visible_alias = "mn",
        value_enum,
        default_value_t = Response::Yes
    )]
    pub name: Response,

    /// Use size when searching.
    ///
    /// Will be overridden to yes if --match-hash is used.
    #[arg(
        long = "match-size",
        visible_alias = "ms",
        value_enum,
        default_value_t = Response::Yes
    )]
    pub size: Response,

    /// Use hash when searching.
    #[arg(
        long = "match-hash",
        visible_alias = "mh",
        value_enum,
        default_value_t = Response::No
    )]
    pub hash: Response,
}

/// Log config.
#[derive(Debug, Args)]
#[command(next_help_heading = "Log")]
pub struct Log {
    /// Log level to use.
    #[arg(long, visible_alias = "ll", default_value_t)]
    log_level: ::log_level_cli::LevelFilter,

    /// File to write log to.
    #[arg(long, visible_alias = "lf", default_value_t = ::patharg::OutputArg::Stdout)]
    log_file: ::patharg::OutputArg,

    /// Print found dupe groups to log.
    #[arg(long)]
    pub log_groups: bool,
}

impl Log {
    /// Initialize log from config.
    ///
    /// # Errors
    /// If log cannot be set up.
    pub fn init(&self) -> Result<(), ::color_eyre::Report> {
        ::env_logger::builder()
            .filter_module(env!("CARGO_CRATE_NAME"), self.log_level.into_inner())
            .parse_env("QUICK_DUPES_LOG")
            .target(match &self.log_file {
                ::patharg::OutputArg::Stdout => ::env_logger::Target::Stderr,
                ::patharg::OutputArg::Path(path_buf) => fs::File::options()
                    .create(true)
                    .append(true)
                    .open(path_buf)
                    .map(|file| ::env_logger::Target::Pipe(Box::new(file)))
                    .map_err(|err| eyre!("failed to open {}", path_buf.display()).error(err))?,
            })
            .init();
        Ok(())
    }
}

/// Find dupes based on matching filenams and hashes.
#[derive(Debug, Parser)]
pub struct Cli {
    /// Directory perform recursice search at.
    #[arg(required = true)]
    pub path: Vec<PathBuf>,

    /// Print dupe groups to stdout.
    ///
    /// Group entries are delimited by either a newline or null character, and split by an empty entry.
    #[arg(long)]
    pub print: bool,

    /// Use null character to separate paths (Useless without --print).
    ///
    /// If specified the default value of --newlines will be include instead of skip.
    #[arg(
        long,
        short = '0',
        visible_short_alias = 'z',
        visible_alias = "print-zero"
    )]
    pub null: bool,

    /// Canonicalize paths.
    #[arg(long, short)]
    pub canonicalize: bool,

    /// How many threads to use.
    #[arg(long, short, default_value_t = default_thread_count())]
    pub threads: NonZero<usize>,

    /// Filter options.
    #[command(flatten)]
    pub filter: Filter,

    /// Log options.
    #[command(flatten)]
    pub log: Log,
}

impl Cli {
    /// Setup cli.
    ///
    /// # Errors
    /// If log or thread pool cannot be set up.
    pub fn setup(mut self) -> Result<Self, ::color_eyre::Report> {
        self.log.init()?;

        if self.canonicalize {
            let mut errors = Vec::new();
            self.path = self
                .path
                .into_iter()
                .filter_map(|path| match path.canonicalize() {
                    Ok(path) => Some(path),
                    Err(err) => {
                        errors.push(CanonicalizationError { path, err });
                        None
                    }
                })
                .collect();

            if !errors.is_empty() {
                return Err(errors
                    .into_iter()
                    .fold(eyre!("could not canonicalize all paths"), |report, err| {
                        report.error(err)
                    }));
            }
        }

        if self.filter.hash.is_yes() {
            self.filter.size = Response::Yes;
        }

        ThreadPoolBuilder::new()
            .num_threads(self.threads.get())
            .thread_name(|idx| format!("quick-dupes-{idx}"))
            .build_global()?;

        Ok(self)
    }
}
