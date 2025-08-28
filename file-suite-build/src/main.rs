#![doc = include_str!("../README.md")]

use ::std::{
    io::{stderr, stdout},
    path::Path,
    process::ExitCode,
};

use ::file_suite_build::tool_json_to_rust;

const HELP: &str = r"usage: file-suite-build TOOL [ARGS...]";

fn help(io: &mut dyn ::std::io::Write) {
    writeln!(io, "{HELP}").unwrap_or_else(|err| panic!("could not write help message, {err}"));
}

fn main() -> ExitCode {
    let args = ::std::env::args().collect::<Vec<_>>();
    let args = args
        .iter()
        .skip(1)
        .map(|arg| arg.as_str())
        .collect::<Vec<_>>();
    let args = &args[..];

    match args {
        ["help"] => {
            help(&mut stdout());
            ExitCode::SUCCESS
        }
        ["tool_json_to_rust", json_file] => {
            println!("{rust}", rust = tool_json_to_rust(Path::new(json_file)));
            ExitCode::SUCCESS
        }
        [] => {
            eprintln!("file-suite-build should not be called without any arguments");
            help(&mut stderr());
            ExitCode::from(2u8)
        }
        other => {
            eprintln!("file-suite-build called with unknown arguments, {other:?}");
            help(&mut stderr());
            ExitCode::from(2u8)
        }
    }
}
