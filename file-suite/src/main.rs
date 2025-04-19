#![doc = include_str!("../README.md")]

use ::std::process::{ExitCode, Termination};

use ::file_suite::Cli;
use ::file_suite_common::{ExitCodeError, start};

fn main() -> ExitCode {
    let Err(err) = start::<Cli>(&[
        "file_suite",
        "path_is_utf8",
        "compile_nested",
        "generate_keyfile",
    ]) else {
        return ExitCode::SUCCESS;
    };

    if let Some(ExitCodeError(exit_code)) = err.downcast_ref::<ExitCodeError>() {
        return *exit_code;
    }

    Termination::report(Err::<(), _>(err))
}
