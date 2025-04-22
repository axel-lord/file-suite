#![doc = include_str!("../README.md")]

use ::std::process::{ExitCode, Termination};

use ::file_suite_common::ExitCodeError;

fn main() -> ExitCode {
    let current = ::std::env::current_exe().ok();
    let cli_name = current
        .as_ref()
        .and_then(|exe| exe.file_name())
        .and_then(|f| f.to_str())
        .unwrap_or("file-suite");

    let (cli_factory, modules) = ::file_suite::get_cli(cli_name);

    let Err(err) = cli_factory().start_as_application(modules) else {
        return ExitCode::SUCCESS;
    };

    if let Some(ExitCodeError(exit_code)) = err.downcast_ref::<ExitCodeError>() {
        return *exit_code;
    }

    Termination::report(Err::<(), _>(err))
}
