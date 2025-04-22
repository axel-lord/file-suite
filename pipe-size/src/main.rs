#![doc = include_str!("../README.md")]

use ::pipe_size::Cli;

/// Application entrypoint.
///
/// # Errors
/// If panic handler cannot be set up.
fn main() -> ::file_suite_common::Result {
    ::file_suite_common::start::<Cli>(&["pipe-size"])
}
