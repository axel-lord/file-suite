//! Tokens used by parser.

use crate::{ByteStr, alias::ByteParser};

/// Token
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ::derive_more::IsVariant)]
pub enum Token<'i> {
    /// Unqouted left parentheses, (.
    LParen,
    /// Unqouted right parentheses, ).
    RParen,
    /// Pipe symbol, |.
    Pipe,
    /// Equals sign, =.
    Eq,
    /// Dash of any length, -.
    Dash(&'i str),
    /// String, 'content'.
    String(&'i ByteStr),
    /// Format string, f"content {value}".
    FString(&'i ByteStr),
    /// Comment, # comment
    Comment(&'i ByteStr),
    /// Identifier surrounded by whitespace, ident.
    Ident(&'i str),
    /// Tabs, line breaks and spaces.
    Whitespace,
}

/// Parser parsing an identifier.
pub fn ident<'i>() -> impl ByteParser<'i, &'i str> + Clone + Copy {
    use ::chumsky::prelude::*;
    any()
        .filter(|b: &u8| b.is_ascii_alphanumeric() || matches!(*b, b'_'))
        .then(
            any()
                .filter(|b: &u8| b.is_ascii_alphanumeric() || matches!(*b, b'_' | b'-' | b'.'))
                .repeated(),
        )
        .to_slice()
        .map(str::from_utf8)
        .unwrapped()
}

impl<'i> Token<'i> {
    /// Get a token parser.
    pub fn parser() -> impl ByteParser<'i, Self> {
        use ::chumsky::prelude::*;

        let sqstring = just(b'\'')
            .ignore_then(none_of(b'\'').repeated().to_slice())
            .then_ignore(just(b'\''))
            .map(ByteStr::new);
        let dqstring = just(b'"')
            .ignore_then(none_of(b'"').repeated().to_slice())
            .then_ignore(just(b'"'))
            .map(ByteStr::new);

        let fstr = |delim: u8| {
            let start = [b'f', delim];
            let esc = [b'\\', delim];

            choice((just(esc).ignored(), none_of(delim).ignored()))
                .repeated()
                .to_slice()
                .delimited_by(just(start), just(delim))
                .map(ByteStr::new)
        };

        let sqfstring = fstr(b'\'');
        let dqfstring = fstr(b'"');

        let comment =
            just(b'#').ignore_then(none_of(b'\n').repeated().to_slice().map(ByteStr::new));
        let ws = one_of(b"\t \n\r").repeated().at_least(1);

        let dash = just(b'-')
            .repeated()
            .at_least(1)
            .to_slice()
            .map(str::from_utf8)
            .unwrapped();

        choice((
            just(b'(').to(Self::LParen),
            just(b')').to(Self::RParen),
            just(b'|').to(Self::Pipe),
            just(b'=').to(Self::Eq),
            dash.map(Self::Dash),
            sqstring.map(Self::String),
            dqstring.map(Self::String),
            sqfstring.map(Self::FString),
            dqfstring.map(Self::FString),
            comment.map(Self::Comment),
            ws.to(Self::Whitespace),
            ident().map(Self::Ident),
        ))
    }
}
