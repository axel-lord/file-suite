#![doc = include_str!("../README.md")]

use ::std::io::{Read, Write};

use ::itertools::{Itertools, MinMaxResult};

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

    /// Use component equality with components split by a value
    /// instead of byte equality.
    #[arg(long, short, default_missing_value = "/", require_equals = true, num_args = 0..=1, value_name = "DELIM")]
    components: Option<String>,
}

fn by_byte_prefix(items: MinMaxResult<&[u8]>) -> ::std::io::Result<()> {
    match items {
        MinMaxResult::NoElements => Err(::std::io::Error::other("input is empty")),
        MinMaxResult::OneElement(line) => ::std::io::stdout().lock().write_all(line),
        MinMaxResult::MinMax(first, last) => {
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

impl ::file_suite_common::Run for Cli {
    type Error = ::std::io::Error;

    fn run(self) -> Result<(), Self::Error> {
        let bytes = {
            let mut buf = Vec::new();
            ::std::io::stdin().lock().read_to_end(&mut buf)?;
            buf
        };

        let delim = if self.null { b'\0' } else { b'\n' };
        let items = bytes.split(|e| *e == delim);

        if let Some(_pat) = self.components {
            todo!()
        } else {
            by_byte_prefix(items.filter(|line| !line.is_empty()).minmax())
        }
    }
}
