#![doc = include_str!("../README.md")]
#![allow(clippy::missing_docs_in_private_items)]

use ::std::fmt::Debug;

use ::clap::{Args, Parser, Subcommand, ValueEnum};
use ::color_eyre::Report;
use ::completions_cli::CompletionConfig;
use ::file_suite_common::{Run, Start, startable};

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

include!(concat!(env!("OUT_DIR"), "/tools.rs"));
