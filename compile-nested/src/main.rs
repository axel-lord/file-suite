#![doc = include_str!("../README.md")]

use ::compile_nested::Cli;
use ::file_suite_common::start;

/// Application entry.
///
/// # Errors
/// If a fatal error occurs or the panic handler cannot be installed.
fn main() -> ::color_eyre::Result<()> {
    start::<Cli>(&["compile_nested"])
}
