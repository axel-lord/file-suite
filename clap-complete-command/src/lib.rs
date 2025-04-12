#![doc = include_str!("../README.md")]

use ::std::{
    io::{BufWriter, Write},
    path::PathBuf,
};

use ::clap::Args;
use ::clap_complete::{generate, Shell};
use ::patharg::OutputArg;

/// Error type returned when failing to generate completions.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Output could not be opened.
    #[error("could not open output [{1:?}], {0}")]
    OpenOutput(::std::io::Error, Option<PathBuf>),

    /// Output could not be flushed.
    #[error("could not flush output [], {0}")]
    FlushOutput(::std::io::Error, Option<PathBuf>),
}

/// Get shell specified by SHELL variable or bash.
fn get_shell() -> Shell {
    Shell::from_env().unwrap_or(Shell::Bash)
}

/// Get binary name.
fn get_binary() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|bin| Some(bin.file_name()?.to_str()?.to_string()))
        .unwrap_or_else(|| String::from(env!("CARGO_CRATE_NAME")))
}

/// Generate command line completions.
#[derive(Debug, Args)]
pub struct Completions {
    /// Shell to generate completions for.
    ///
    /// Defaults to shell specified by the SHELL environment
    /// variable or bash if not available.
    #[arg(default_value_t = get_shell())]
    shell: Shell,

    /// What name to use for binary.
    #[arg(long, short, default_value_t = get_binary())]
    binary_name: String,

    /// File to save completions to, if not specified stdout is used.
    #[arg(long, short, default_value_t)]
    output: OutputArg,
}

impl Completions {
    /// Generate completions using provided options.
    ///
    /// # Errors
    /// If the file cannot be opened.
    pub fn generate(self, command: &mut ::clap::Command) -> Result<(), Error> {
        let mut file = self
            .output
            .create()
            .map_err(|err| Error::OpenOutput(err, self.output.clone().into_path()))?
            .map_right(BufWriter::new);

        generate(self.shell, command, self.binary_name, &mut file);

        file.flush()
            .map_err(|err| Error::FlushOutput(err, self.output.clone().into_path()))?;

        Ok(())
    }
}
