#![doc = include_str!("../README.md")]

use ::std::process::{ExitCode, Termination};

use ::file_suite_common::{start, ExitCodeError};
use ::path_is_utf8::Cli;

/// Application entrypoint.
///
/// # Errors
/// On failure.
fn main() -> ExitCode {
    let Err(err) = start::<Cli>(&["path_is_utf8"]) else {
        return ExitCode::SUCCESS;
    };

    if let Some(ExitCodeError(exit_code)) = err.downcast_ref::<ExitCodeError>() {
        return *exit_code;
    }

    Termination::report(Err::<(), _>(err))
}
