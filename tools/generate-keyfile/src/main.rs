#![doc = include_str!("../README.md")]

use ::file_suite_common::start;
use ::generate_keyfile::Cli;

/// Application entrypoint.
///
/// # Errors
/// If any io should fail.
fn main() -> ::file_suite_common::Result {
    start::<Cli>(&["generate_keyfile"])
}
