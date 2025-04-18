//! Crate to recursively rename directories to lowercase.

use ::std::path::PathBuf;

use ::clap::Parser;

/// Recursively rename files to lowercase.
#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
pub struct Cli {
    /// Log file renames and no-ops as INFO.
    #[arg(short, long)]
    verbose: bool,

    /// File/Directory to start at.
    #[arg(required = true)]
    file: Vec<PathBuf>,
}
