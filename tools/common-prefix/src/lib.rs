#![doc = include_str!("../README.md")]

use ::std::io::{Read, Write};

/// Find the common prefix of all lines piped to stdin.
///
/// Empty lines are ignored.
#[derive(Debug, ::clap::Parser)]
pub struct Cli {
    /// Input is separated by null characters.
    #[arg(
        short = '0',
        long = "null",
        visible_alias = "print0",
        visible_short_alias = 'z'
    )]
    null: bool,
}

impl ::file_suite_common::Run for Cli {
    type Error = ::std::io::Error;

    fn run(self) -> Result<(), Self::Error> {
        let bytes = {
            let mut buf = Vec::new();
            ::std::io::stdin().lock().read_to_end(&mut buf)?;
            buf
        };

        let delim = if self.null { b'\0' } else { b'\n' };
        let mut lines = bytes
            .split(|e| *e == delim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>();

        lines.sort_unstable();

        match lines.as_slice() {
            [] => Err(::std::io::Error::other("input is empty")),
            [line] => ::std::io::stdout().lock().write_all(line),
            [first, .., last] => {
                let prefix = {
                    let mut prefix = b"".as_slice();
                    for i in 0.. {
                        let Some(initial) = first.get(0..i) else {
                            break;
                        };
                        if !last.starts_with(initial) {
                            break;
                        }
                        prefix = initial;
                    }
                    prefix
                };

                ::std::io::stdout().lock().write_all(prefix)
            }
        }
    }
}
