#![doc = include_str!("../README.md")]

/// command line interface for arg-shell.
#[derive(Debug, ::clap::Parser)]
pub struct Cli {}

impl ::file_suite_common::Run for Cli {
    type Error = ::std::convert::Infallible;

    fn run(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
