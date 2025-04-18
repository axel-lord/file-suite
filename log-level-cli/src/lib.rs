#![doc = include_str!("../README.md")]

use ::std::{fmt::Display, fs::File, io::BufWriter};

use ::clap::{Args, ValueEnum, builder::PossibleValue};

/// Cli options to configure logging.
#[derive(Debug, Args)]
pub struct LogConfig {
    /// Where to output log, when -, stderr is used.
    #[arg(long, visible_alias = "lf", default_value = "-")]
    pub log_file: OutputArg,
    /// At what level to log.
    #[arg(long, visible_alias = "ll", default_value_t)]
    pub log_level: LevelFilter,
}

impl LogConfig {
    /// Install configured logger.
    ///
    /// # Panics
    /// If a logger has already been initialized or if the output file cannot be opened for writing/appending.
    pub fn init<I, S>(self, modules: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        modules
            .into_iter()
            .fold(&mut ::env_logger::builder(), |builder, module| {
                builder.filter_module(module.as_ref(), self.log_level.into_inner())
            })
            .target(match self.log_file {
                OutputArg::Stdout => ::env_logger::Target::Stderr,
                OutputArg::Path(path_buf) => ::env_logger::Target::Pipe(Box::new(BufWriter::new(
                    File::options()
                        .append(true)
                        .create(true)
                        .open(&path_buf)
                        .unwrap_or_else(|err| {
                            panic!(
                                "could not open log file, '{path_buf}', {err}",
                                path_buf = path_buf.display()
                            )
                        }),
                ))),
            })
            .init();
    }
}

/// Level newtype.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Level(pub ::log::Level);

impl Default for Level {
    fn default() -> Self {
        Self(if cfg!(debug_assertions) {
            ::log::Level::Trace
        } else {
            ::log::Level::Info
        })
    }
}

impl_value_enum!(Level(::log::Level), Trace, Debug, Info, Warn, Error);

/// LevelFilter newtype.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LevelFilter(pub ::log::LevelFilter);

impl Default for LevelFilter {
    fn default() -> Self {
        Self(if cfg!(debug_assertions) {
            ::log::LevelFilter::Trace
        } else {
            ::log::LevelFilter::Info
        })
    }
}

impl_value_enum!(
    LevelFilter(::log::LevelFilter),
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off
);

/// Implement Display and ValueEnum for LogLevel.
macro_rules! impl_value_enum {
    ($nm:ident($ty:ty), $($var:ident),*) => {
        paste::paste! {
        impl $nm {
            #[doc = concat!("Convert to internal [", stringify!($ty), "]")]
            pub const fn into_inner(self) -> $ty {
                self.0
            }
        }
        }

        impl ValueEnum for $nm {
            fn value_variants<'a>() -> &'a [Self] {
                &[$(
                    Self(<$ty> :: $var),
                )*]
            }

            fn to_possible_value(&self) -> Option<PossibleValue> {
                paste::paste! {
                Some(match self.0 {$(
                    $ty::$var => PossibleValue::new(stringify!($var)).aliases([
                        stringify!([< $var:lower >]), stringify!([< $var:upper >])
                    ]),
                )*})
                }
            }
        }

        impl Display for $nm {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self.0 {$(
                    <$ty>::$var => write!(f, "{}", stringify!($var) ),
                )*}
            }
        }

        impl From<$nm> for $ty {
            fn from(value: $nm) -> $ty {
                value.into_inner()
            }
        }

        impl From<$ty> for $nm {
            fn from(value: $ty) -> $nm {
                $nm(value)
            }
        }
    };
}
use ::patharg::OutputArg;
use impl_value_enum;
