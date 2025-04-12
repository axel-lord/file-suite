use ::std::{fs, num::NonZero, path::PathBuf, thread};

use ::clap::{builder::ArgPredicate::Equals, Args, Parser, ValueEnum};
use ::color_eyre::{eyre::eyre, Section};
use ::derive_more::IsVariant;
use ::rayon::ThreadPoolBuilder;

use crate::error::CanonicalizationError;

fn default_thread_count() -> NonZero<usize> {
    thread::available_parallelism()
        .unwrap_or(const { NonZero::new(1).expect("1 should not equal 0") })
}

#[derive(Clone, Copy, Debug, ValueEnum, IsVariant)]
pub enum NewLineBehaviour {
    #[value(alias = "s")]
    Skip,
    #[value(alias = "i")]
    Include,
}

#[derive(Clone, Copy, Debug, ValueEnum, IsVariant)]
pub enum Response {
    #[value(alias = "y")]
    Yes,
    #[value(alias = "n")]
    No,
}

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

#[derive(Debug, Args)]
#[command(next_help_heading = "Log")]
pub struct Log {
    /// Log level to use.
    #[arg(long, visible_alias = "ll", default_value_t)]
    log_level: ::clap_log_level::LevelFilter,

    /// File to write log to.
    #[arg(long, visible_alias = "lf", default_value_t = ::patharg::OutputArg::Stdout)]
    log_file: ::patharg::OutputArg,

    /// Print found dupe groups to log.
    #[arg(long)]
    pub log_groups: bool,
}

impl Log {
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

    #[command(flatten)]
    pub filter: Filter,

    #[command(flatten)]
    pub log: Log,
}

impl Cli {
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
