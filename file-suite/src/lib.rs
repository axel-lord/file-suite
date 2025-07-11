#![doc = include_str!("../README.md")]
#![allow(clippy::missing_docs_in_private_items)]

use ::std::fmt::Debug;

use ::clap::{Args, Parser, Subcommand, ValueEnum};
use ::color_eyre::Report;
use ::completions_cli::CompletionConfig;
use ::file_suite_common::{Run, Start, startable};
use ::file_suite_proc::array_expr;

/// Application for containing an amount of file-system related utilities.
#[derive(Debug, Parser, Run)]
#[run(error = Report)]
#[command(author, version, long_about = None)]
pub struct Cli {
    /// Tool to use.
    #[command(subcommand)]
    subcmd: CliSubcmd,
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
    type Error = Report;

    fn run(self) -> Result<(), Self::Error> {
        self.completion_config
            .generate(&mut self.tool.startable().command_as_application(), || {
                self.tool.mod_name().replace("_", "-")
            })
            .map_err(Report::from)
    }
}

array_expr! {
    generate_keyfile compile_nested path_is_utf8 pipe_size -> global(modules),

    snakeToPascal -> alias { split(snake).case(pascal).join.ty(ident) },
    snakeToKebab -> alias { split(snake).case(lower).join(kebab).ty(str) },

    =modules -> ty(str).global(strModules),
    =modules -> for_each(=snakeToKebab).global(kebabModules),
    =modules -> for_each(=snakeToPascal).global(pascalModules),

    -> .paste {
        /// Modules to allow logging for.
        pub const MODULES: &[&str] = &[ "file_suite" #(, #strModules)* ];

        /// Get cli and used modules from tool name.
        pub fn get_cli(name: &str) -> (fn() -> &'static dyn Start, &'static [&'static str]) {
            // Quick path for compilation tool.
            if name == "file-suite" {
                return (|| startable::<Cli>(), MODULES);
            }
            match name {
                #( #kebabModules => (|| startable::<::#modules::Cli>(), &[ #strModules ]), )*
                _ => (|| startable::<Cli>(), MODULES),
            }
        }

        // Selection of cli tool.
        #[derive(Debug, Subcommand, Run)]
        #[run(error = Report)]
        enum CliSubcmd {
            // generate completions for a tool
            Completions(CmpSubcmd),
            #( #pascalModules(::#modules::Cli), )*
        }

        /// Module to generate completions for
        #[derive(Debug, ValueEnum, Clone, Copy, PartialEq, Eq, Hash, Default)]
        enum CompletionTarget {
            #[default]
            FileSuite,
            #( #pascalModules, )*
        }

        impl CompletionTarget {
            /// Get a startable based on the selected tool.
            fn startable(self) -> &'static dyn Start {
                match self {
                    Self::FileSuite => startable::<Cli>(),
                    #(
                    Self::  #pascalModules => startable::<::#modules::Cli>(),
                    )*
                }
            }

            /// Get name of startable.
            const fn mod_name(self) -> &'static str {
                match self {
                    Self::FileSuite => "file_suite",
                    #(
                    Self:: #pascalModules => #strModules,
                    )*
                }
            }
        }

    },
}
