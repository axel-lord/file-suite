//! Format string exec.

use crate::{ByteStr, alias::ByteParser, smallvec::SmallVec, token::ident};

/// A Fragment of an fstring.
#[derive(Debug, Clone, Copy)]
pub enum FStringFragment<'i> {
    /// Paste text.
    Text(&'i ByteStr),
    /// Lookup variable.
    Lookup(&'i str),
}

impl<'i> FStringFragment<'i> {
    /// Create an fstring fragment parser.
    pub fn parser() -> impl ByteParser<'i, Self> + Clone + Copy {
        use ::chumsky::prelude::*;
        use FStringFragment::{Lookup, Text};

        choice((
            just(b"{{").to(Text(ByteStr::new(b"{"))),
            just(b"}}").to(Text(ByteStr::new(b"}"))),
            just(br"\").to(Text(ByteStr::new(br"\"))),
            just(br#"\""#).to(Text(ByteStr::new(br#"""#))),
            just(br"\'").to(Text(ByteStr::new(b"'"))),
            just(br"\n").to(Text(ByteStr::new(b"\n"))),
            just(br"\t").to(Text(ByteStr::new(b"\t"))),
            just(br"\r").to(Text(ByteStr::new(b"\r"))),
            ident().delimited_by(just(b'{'), just(b'}')).map(Lookup),
            none_of(br"{}\")
                .repeated()
                .at_least(1)
                .to_slice()
                .map(ByteStr::new)
                .map(Text),
        ))
    }
}

/// A format string.
#[derive(Debug, Clone)]
pub struct FString<'i>(pub SmallVec<3, FStringFragment<'i>>);

impl<'i> FString<'i> {
    /// Create an fstring parser.
    pub fn parser() -> impl ByteParser<'i, Self> + Clone + Copy {
        use ::chumsky::prelude::*;
        FStringFragment::parser().repeated().collect().map(Self)
    }
}
