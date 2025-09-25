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
    #[arg(
        short = '0',
        long = "null",
        visible_alias = "print0",
        visible_short_alias = 'z'
    )]
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Component<'b> {
    pub value: &'b [u8],
    pub sep: Option<&'b [u8]>,
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

    fn cmp_multiple(a: &[Self], b: &[Self]) -> Ordering {
        let a = a.iter().map(|comp| comp.value);
        let b = b.iter().map(|comp| comp.value);
        ::std::iter::Iterator::cmp(a, b)
    }
}

fn by_component<'a>(items: impl Iterator<Item = &'a [u8]>, re: Regex) {
    let items = items
        .map(|item| Component::parse_all(&re, item).collect::<Vec<_>>())
        .minmax_by(|a, b| Component::cmp_multiple(a, b));
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
