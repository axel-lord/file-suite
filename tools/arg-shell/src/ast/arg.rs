//! Command argument ast types.

use ::enum_dispatch::enum_dispatch;

use crate::{
    ByteStr, alias::ByteParser, ast::Ast, smallvec::SmallVec, token::ident, withspan::WithSpan,
};

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

/// Arguments for calls.
#[enum_dispatch(Argument)]
#[derive(Debug, Clone)]
pub enum Arg<'i> {
    /// Pass string as is.
    String(WithSpan<&'i ByteStr>),
    /// A format string.
    FString(FString<'i>),
    /// Group as an argument.
    Group(Ast<'i>),
}
