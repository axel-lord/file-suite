//! Main function.

use ::clap::Parser;
use ::recursive_lowercase::Cli;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    Cli::parse().run();
    Ok(())
}
