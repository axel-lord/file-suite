//! Parse format strings.

use ::std::{
    borrow::Cow,
    error::Error,
    fmt::{Debug, Display},
};

use ::chumsky::{Parser, error::Rich, extra::ParserExtra};

pub mod lookup;

/// Get a debug implementor for a byte array representing text.
pub fn debug_bytes<'a>(bytes: &'a [u8]) -> impl 'a + Debug {
    struct DebugU8Slice<'b>(&'b [u8]);
    impl Debug for DebugU8Slice<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "b\"")?;
            for chunk in self.0.utf8_chunks() {
                for chr in chunk.valid().chars() {
                    write!(f, "{}", chr.escape_debug())?;
                }

                for b in chunk.invalid() {
                    write!(f, "\\x{b:02X}")?;
                }
            }
            write!(f, "\"")?;
            Ok(())
        }
    }
    DebugU8Slice(bytes)
}
/// Get a display implementor for a byte array representing text.
pub fn display_bytes<'a>(bytes: &'a [u8]) -> impl 'a + Display {
    struct DisplayU8Slice<'b>(&'b [u8]);
    impl Display for DisplayU8Slice<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for chunk in self.0.utf8_chunks() {
                for chr in chunk.valid().chars() {
                    write!(f, "{}", chr.escape_debug())?;
                }

                for b in chunk.invalid() {
                    write!(f, "\\x{b:02X}")?;
                }
            }
            Ok(())
        }
    }
    DisplayU8Slice(bytes)
}

/// A part of the format string.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Chunk<'a> {
    /// Text contained by `{}`.
    Lookup(&'a [u8]),
    /// Text outside `{}`.
    Text(&'a [u8]),
}

impl Debug for Chunk<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lookup(arg0) => f.debug_tuple("Lookup").field(&debug_bytes(arg0)).finish(),
            Self::Text(arg0) => f.debug_tuple("Text").field(&debug_bytes(arg0)).finish(),
        }
    }
}

impl<'a> Chunk<'a> {
    /// Resolve the chunk returning either the text as is, or the result of a lookup.
    pub fn resolve<E>(
        self,
        lookup: impl FnOnce(&'a [u8]) -> Result<Cow<'a, [u8]>, E>,
    ) -> Result<Cow<'a, [u8]>, E> {
        match self {
            Chunk::Lookup(items) => lookup(items),
            Chunk::Text(items) => Ok(Cow::Borrowed(items)),
        }
    }

    /// Get a parser for a chunk.
    pub fn parser<E: ParserExtra<'a, &'a [u8]>>() -> impl Parser<'a, &'a [u8], Self, E> {
        use ::chumsky::prelude::*;

        let as_is = none_of(b"{}")
            .repeated()
            .at_least(1)
            .to_slice()
            .map(Chunk::Text);

        let lookup = none_of(b"{}")
            .repeated()
            .to_slice()
            .map(Chunk::Lookup)
            .delimited_by(just(b'{'), just(b'}'));

        choice((
            as_is,
            just(b"{{").to(Chunk::Text(b"{")),
            just(b"}}").to(Chunk::Text(b"}")),
            lookup,
        ))
    }
}
/// Create an iterator of chunks from a format string.
pub fn parse_fmt<'a>(
    fmt: &'a [u8],
) -> impl 'a + Iterator<Item = Result<Chunk<'a>, Vec<Rich<'a, u8>>>> {
    use ::chumsky::prelude::*;
    let parser =
        Chunk::parser::<::chumsky::extra::Err<Rich<u8>>>().then(any().repeated().to_slice());
    let mut fmt = fmt;
    ::std::iter::from_fn(move || {
        if fmt.is_empty() {
            return None;
        }
        match parser.parse(fmt).into_result() {
            Ok((chunk, remainder)) => {
                fmt = remainder;
                Some(Ok(chunk))
            }
            Err(err) => Some(Err(err)),
        }
    })
}

/// Use an iterator of chunks to write formatted bytes to a container.
pub fn format_to<'a, C, I, E, F>(buf: &mut C, chunks: I, mut lookup: F) -> Result<(), E>
where
    I: IntoIterator<Item = Chunk<'a>>,
    C: Extend<u8>,
    F: FnMut(&'a [u8]) -> Result<Cow<'a, [u8]>, E>,
{
    for chunk in chunks {
        let bytes = chunk.resolve(&mut lookup)?;
        buf.extend(bytes.into_iter().copied());
    }
    Ok(())
}

/// Parse a format string and format using provided lookup funtion.
pub fn format<'a, C, E, F>(fmt: &'a [u8], mut lookup: F) -> Result<C, FormatError<'a, E>>
where
    C: Extend<u8> + Default,
    F: FnMut(&'a [u8]) -> Result<Cow<'a, [u8]>, E>,
{
    let mut c = C::default();
    for chunk in parse_fmt(fmt) {
        let chunk = chunk.map_err(FormatError::Parse)?;
        let bytes = chunk.resolve(&mut lookup).map_err(FormatError::Lookup)?;
        c.extend(bytes.iter().copied());
    }

    Ok(c)
}

/// Error returned when formatting fails, may be
/// either due to an incorrect format string, or
/// due to a failed lookup.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum FormatError<'a, E> {
    /// Format string could not be parsed.
    Parse(Vec<Rich<'a, u8>>),
    /// Lookup failed.
    Lookup(E),
}

impl<'a, E> Error for FormatError<'a, E>
where
    E: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FormatError::Parse(_) => None,
            FormatError::Lookup(e) => Some(e),
        }
    }
}

impl<'a, E> Display for FormatError<'a, E>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::Parse(items) => match items.as_slice() {
                [] => {
                    write!(f, "unknown parse error")
                }
                [head @ .., tail] => {
                    for err in head {
                        <Rich<u8> as Display>::fmt(err, f)?;
                        writeln!(f)?;
                    }
                    <Rich<u8> as Display>::fmt(tail, f)
                }
            },
            FormatError::Lookup(err) => <E as Display>::fmt(err, f),
        }
    }
}

impl<'a, E> FormatError<'a, E> {
    /// Map the lookup variant.
    pub fn map_lookup_err<T, F>(self, f: F) -> FormatError<'a, T>
    where
        F: FnOnce(E) -> T,
    {
        match self {
            FormatError::Parse(items) => FormatError::Parse(items),
            FormatError::Lookup(e) => FormatError::Lookup(f(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use ::std::collections::HashMap;

    use crate::lookup::SeqLookupError;

    use super::*;
    use ::chumsky::{IterParser, extra};
    use ::pretty_assertions::assert_eq;
    use Chunk::{Lookup, Text};

    #[test]
    fn single_chunk() {
        let parser = Chunk::parser::<extra::Default>();

        let expected = Text(b"Hello world!");
        let result = parser.parse(b"Hello world!").into_result();
        assert_eq!(result, Ok(expected));

        let expected = Text(b"{");
        let result = parser.parse(b"{{").into_result();
        assert_eq!(result, Ok(expected));

        let expected = Text(b"}");
        let result = parser.parse(b"}}").into_result();
        assert_eq!(result, Ok(expected));

        let expected = Lookup(b"key-value");
        let result = parser.parse(b"{key-value}").into_result();
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn multiple_chunks() {
        let parser = Chunk::parser::<extra::Default>().repeated().collect();

        let expected = vec![
            Text(b"{"),
            Text(b"Key: "),
            Lookup(b"key"),
            Text(b", Value: "),
            Lookup(b"value"),
            Text(b"}"),
        ];
        let result = parser
            .parse(b"{{Key: {key}, Value: {value}}}")
            .into_result();
        assert_eq!(result, Ok(expected));

        let expected = vec![
            Lookup(b""),
            Text(b", "),
            Lookup(b""),
            Text(b", "),
            Lookup(b""),
        ];
        let result = parser.parse(b"{}, {}, {}").into_result();
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn iter() {
        let mut iter = parse_fmt(b"{}, {}, {key}, end}}");

        for _ in 0..2 {
            assert_eq!(iter.next(), Some(Ok(Lookup(b""))));
            assert_eq!(iter.next(), Some(Ok(Text(b", "))));
        }

        assert_eq!(iter.next(), Some(Ok(Lookup(b"key"))));
        assert_eq!(iter.next(), Some(Ok(Text(b", end"))));
        assert_eq!(iter.next(), Some(Ok(Text(b"}"))));

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn format() {
        use super::format;
        let values: [(&[u8], &[u8]); 4] = [
            (b"HOME", b"/var/home/user"),
            (b"ZERO", b"0"),
            (b"key", b"lock"),
            (b"passwd", b"1234"),
        ];
        let value_map = HashMap::from(values);

        assert_eq!(
            format(b"password: {passwd}", lookup::hash_map(&value_map)),
            Ok(Vec::from(b"password: 1234"))
        );

        assert_eq!(
            format(b"number: {ZERO}", lookup::seq_map(&values)),
            Ok(Vec::from(b"number: 0"))
        );

        assert_eq!(
            format::<Vec<u8>, _, _>(b"number: {ONE}", lookup::seq_map(&values)),
            Err(FormatError::Lookup(b"ONE".as_slice()))
        );

        let seq: [&[u8]; 3] = [b"ZERO", b"ONE", b"TWO"];

        assert_eq!(
            format(
                b"0: {}, 1: {}, 2: {}, 1: {1}, 0: {0}, last: {-1}",
                lookup::seq(&seq)
            ),
            Ok(Vec::from(
                b"0: ZERO, 1: ONE, 2: TWO, 1: ONE, 0: ZERO, last: TWO"
            ))
        );

        assert_eq!(
            format::<Vec<u8>, _, _>(
                b"0: {}, 1: {}, 2: {}, 1: {1}, 0: {0}, last: {-1}, next: {}",
                lookup::seq(&seq)
            ),
            Err(FormatError::Lookup(SeqLookupError::OutOfRange(3, -3..3)))
        );
    }
}
