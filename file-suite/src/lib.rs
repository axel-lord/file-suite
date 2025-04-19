#![doc = include_str!("../README.md")]

use ::clap::Parser;
use ::file_suite_common::Run;

/// Application for containing an amount of file-system related utilities.
#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
pub struct Cli {}

impl Run for Cli {
    type Err = ::color_eyre::Report;

    fn run(self) -> Result<(), Self::Err> {
        Ok(())
    }
}
