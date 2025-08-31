//! Completion config.

use ::std::io::{BufWriter, Write};

use ::clap::{Args, CommandFactory};
use ::clap_complete::Shell;
use ::tap::Pipe;

/// Get default shell.
fn get_shell() -> Shell {
    Shell::from_env().unwrap_or(Shell::Bash)
}

/// Get default binary name.
fn get_bin() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|name| name.file_name()?.to_str()?.to_string().pipe(Some))
        .unwrap_or_else(|| String::from(env!("CARGO_BIN_NAME")))
}

/// Completion arguments.
#[derive(Debug, Args)]
#[command(next_help_heading = "Completion Options")]
pub struct Completions {
    /// Output completions.
    #[arg(long, hide_short_help = true)]
    pub completions: bool,

    /// Set shell to generate completions for.
    #[arg(long, default_value_t = get_shell(), requires = "completions", hide_short_help = true)]
    shell: Shell,

    /// Set name of binary to generate completions for.
    #[arg(long, default_value_t = get_bin(), requires = "completions", hide_short_help = true)]
    binary: String,
}

impl Completions {
    /// Generate completions.
    ///
    /// # Errors
    /// If file written to cannot be flushed.
    pub fn generate(self, to: patharg::OutputArg) -> ::color_eyre::Result<()> {
        let mut file = to.create()?.map_right(BufWriter::new);

        clap_complete::generate(
            self.shell,
            &mut crate::Cli::command(),
            self.binary,
            &mut file,
        );
        file.flush()?;
        Ok(())
    }
}
