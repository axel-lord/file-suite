#![doc = include_str!("../README.md")]

use ::std::{
    cmp::Ordering,
    io::{Read, Write},
};

use ::itertools::{Itertools, MinMaxResult};
use ::regex::bytes::{Regex, RegexBuilder};

fn compile_re(pattern: &str) -> Result<Regex, ::regex::Error> {
    RegexBuilder::new(pattern).multi_line(true).build()
}

/// Find the common prefix of all lines piped to stdin.
///
/// Empty lines are ignored.
#[derive(Debug, ::clap::Parser)]
pub struct Cli {
    /// Input is separated by null characters.
    #[arg(short = '0', long, visible_alias = "print0", visible_short_alias = 'z')]
    null: bool,

    /// Use component equality with components split by a value
    /// instead of byte equality.
    #[arg(
        long,
        short,
        default_missing_value = "/",
        value_parser = compile_re,
        require_equals = true,
        num_args = 0..=1,
        value_name = "DELIM_PATTERN"
    )]
    components: Option<Regex>,

    /// After finding the prefix, print all non-empty input lines and them without the prefix
    /// separated by newlines or null bytes depending on the 'null' option.
    #[arg(long, visible_alias = "pairs")]
    print_pairs: bool,
}

fn by_byte_prefix(items: MinMaxResult<&[u8]>) -> ::std::io::Result<&[u8]> {
    match items {
        MinMaxResult::NoElements => Err(::std::io::Error::other("input is empty")),
        MinMaxResult::OneElement(line) => Ok(line),
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

            Ok(prefix)
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Component<'b> {
    pub value: &'b [u8],
    pub sep: Option<&'b [u8]>,
}

impl<'b> AsRef<Component<'b>> for Component<'b> {
    fn as_ref(&self) -> &Component<'b> {
        self
    }
}

impl<'b> Component<'b> {
    fn parse(re: &Regex, bytes: &'b [u8], at: usize) -> (Self, usize) {
        if let Some(mat) = re.find_at(bytes, at) {
            let value = &bytes[at..mat.start()];
            let sep = Some(mat.as_bytes());
            (Self { value, sep }, mat.end())
        } else {
            let value = &bytes[at..];
            let sep = None;
            (Self { value, sep }, bytes.len())
        }
    }

    fn parse_all(re: &'b Regex, bytes: &'b [u8]) -> impl 'b + Iterator<Item = Self> {
        let mut at = 0;
        ::std::iter::from_fn(move || {
            if at >= bytes.len() {
                return None;
            }
            let comp;
            (comp, at) = Self::parse(re, bytes, at);
            Some(comp)
        })
    }

    fn write_all(
        comps: impl IntoIterator<Item = impl AsRef<Self>>,
        mut w: impl Write,
    ) -> ::std::io::Result<()> {
        for comp in comps {
            w.write_all(comp.as_ref().value)?;
            if let Some(sep) = comp.as_ref().sep {
                w.write_all(sep)?;
            }
        }
        Ok(())
    }

    fn cmp_multiple(a: &[Self], b: &[Self]) -> Ordering {
        let a = a.iter().map(|comp| comp.value);
        let b = b.iter().map(|comp| comp.value);
        ::std::iter::Iterator::cmp(a, b)
    }
}

fn by_component<'a>(
    items: impl IntoIterator<Item = impl AsRef<[Component<'a>]>>,
    w: impl Write,
) -> ::std::io::Result<usize> {
    let items = items
        .into_iter()
        .minmax_by(|a, b| Component::cmp_multiple(a.as_ref(), b.as_ref()));

    match items {
        MinMaxResult::NoElements => Err(::std::io::Error::other("input is empty")),
        MinMaxResult::OneElement(elem) => {
            Component::write_all(elem.as_ref(), w)?;
            Ok(elem.as_ref().len())
        }
        MinMaxResult::MinMax(first, last) => {
            let prefix = {
                let mut prefix: &[Component] = &[];
                for i in 0.. {
                    let Some(initial) = first.as_ref().get(0..i) else {
                        break;
                    };
                    if !last.as_ref().starts_with(initial) {
                        break;
                    }
                    prefix = initial;
                }
                prefix
            };
            Component::write_all(prefix, w)?;
            Ok(prefix.as_ref().len())
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
        let items = bytes.split(|e| *e == delim).filter(|line| !line.is_empty());

        if let Some(pat) = self.components {
            let items = items.map(|bytes| Component::parse_all(&pat, bytes).collect::<Vec<_>>());

            if self.print_pairs {
                let items = items.collect::<Vec<_>>();
                let mut stdout = ::std::io::stdout().lock();
                let len = by_component(&items, &mut stdout)?;

                for item in items {
                    stdout.write_all(&[delim])?;
                    Component::write_all(&item, &mut stdout)?;
                    stdout.write_all(&[delim])?;
                    Component::write_all(&item[len..], &mut stdout)?;
                }

                Ok(())
            } else {
                by_component(items, ::std::io::stdout().lock())?;
                Ok(())
            }
        } else {
            if self.print_pairs {
                let items = items.collect::<Vec<_>>();
                let prefix = by_byte_prefix(items.iter().copied().minmax())?;
                let len = prefix.len();

                let mut stdout = ::std::io::stdout().lock();
                stdout.write_all(prefix)?;

                for item in items {
                    stdout.write_all(&[delim])?;
                    stdout.write_all(item)?;
                    stdout.write_all(&[delim])?;
                    stdout.write_all(&item[len..])?;
                }

                Ok(())
            } else {
                let prefix = by_byte_prefix(items.minmax())?;
                ::std::io::stdout().lock().write_all(prefix)
            }
        }
    }
}
