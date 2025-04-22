#![doc = include_str!("../README.md")]

use ::std::fmt::Debug;

use ::clap::{Args, Parser, Subcommand, ValueEnum};
use ::color_eyre::Report;
use ::completions_cli::CompletionConfig;
use ::file_suite_common::{Run, Start, startable};
use ::file_suite_proc::kebab_paste;
use ::paste::paste;

subcmd!(generate_keyfile, compile_nested, path_is_utf8, pipe_size);

/// Application for containing an amount of file-system related utilities.
#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
pub struct Cli {
    /// Tool to use.
    #[command(subcommand)]
    subcmd: CliSubcmd,
}

impl Run for Cli {
    type Err = Report;

    fn run(self) -> Result<(), Self::Err> {
        self.subcmd.run()
    }
}

/// Completion generation.
#[derive(Debug, Args)]
struct CmpSubcmd {
    /// What tool to generate completions for.
    #[arg(value_enum, default_value_t)]
    tool: CompletionTarget,

    /// Completion config.
    #[command(flatten)]
    completion_config: CompletionConfig,
}

impl Run for CmpSubcmd {
    type Err = Report;

    fn run(self) -> Result<(), Self::Err> {
        self.completion_config
            .generate(&mut self.tool.startable().command_as_application(), || {
                self.tool.mod_name().replace("_", "-")
            })
            .map_err(Report::from)
    }
}

/// Define subcommand.
macro_rules! subcmd {
    ($($mod:ident),* $(,)?) => {
        #[doc = "Modules to allow logging for."]
        pub const MODULES: &[&str] = &["file_suite" $(, stringify!($mod))*];

        kebab_paste! {

        #[doc = "Get cli and used modules from tool name."]
        pub fn get_cli(name: &str) -> (fn() -> &'static dyn Start, &'static [&'static str]) {
            // Quick path for compilation tool.
            if name == "file-suite" {
                return (|| startable::<$crate::Cli>(), MODULES);
            }
            match name {
                $(
                -!($mod[snake] -> str[kebab]) => (|| startable::<::$mod::Cli>(), &[-!($mod -> str)]),
                )*
                _ => (|| startable::<$crate::Cli>(), MODULES),
            }
        }

        #[doc = "Selection of cli tool."]
        #[derive(Debug, Subcommand)]
        enum CliSubcmd {
            #[doc = "generate completions for a tool."]
            Completions(CmpSubcmd),
            $(
            -!($mod[snake] -> [pascal])(::$mod::Cli),
            )*
        }

        }

        paste! {

        impl Run for CliSubcmd {
            type Err = Report;

            fn run(self) -> Result<(), Self::Err> {
                match self {
                    Self::Completions(sub) => sub.run()?,
                    $(
                    Self:: [< $mod:camel >](sub) => sub.run()?,
                    )*
                };
                Ok(())
            }
        }

        #[doc = "Module to generate completions for"]
        #[derive(Debug, ValueEnum, Clone, Copy, PartialEq, Eq, Hash, Default)]
        enum CompletionTarget {
            #[default]
            FileSuite,
            $(
            [< $mod:camel >],
            )*
        }

        impl CompletionTarget {
            /// Get a startable based on the selected tool.
            fn startable(self) -> &'static dyn Start {
                match self {
                    Self::FileSuite => startable::<Cli>(),
                    $(
                    Self::[< $mod:camel >] => startable::<::$mod::Cli>(),
                    )*
                }
            }

            /// Get name of startable.
            const fn mod_name(self) -> &'static str {
                match self {
                    Self::FileSuite => "file_suite",
                    $(
                    Self::[< $mod:camel >] => stringify!($mod),
                    )*
                }
            }
        }

        }
    };
}
use subcmd;
