use ::chumsky::{Parser, extra};

use crate::ByteStr;

type Extra<'i> = extra::Err<::chumsky::error::Rich<'i, u8>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Token<'i> {
    /// Unqouted left parentheses, (.
    LParen,
    /// Unqouted right parentheses, ).
    RParen,
    /// Pipe symbol, |.
    Pipe,
    /// String, 'content'.
    String(&'i ByteStr),
    /// Format string, f"content {value}".
    FString(&'i ByteStr),
    /// Comment, # comment
    Comment(&'i ByteStr),
    /// Identifier surrounded by whitespace, ident.
    Ident(&'i ByteStr),
    /// Terminating whitespace, linebreaks and colons.
    Term,
    /// Tabs and spaces.
    Whitespace,
}

impl<'i> Token<'i> {
    pub fn parser() -> impl Parser<'i, &'i [u8], Self, Extra<'i>> {
        use ::chumsky::prelude::*;

        let sqstring = just(b'\'')
            .ignore_then(none_of(b'\'').repeated().to_slice())
            .then_ignore(just(b'\''))
            .map(ByteStr::new);
        let dqstring = just(b'"')
            .ignore_then(none_of(b'"').repeated().to_slice())
            .then_ignore(just(b'"'))
            .map(ByteStr::new);

        let sqfstring = just(b"f'")
            .ignore_then(
                just(b"\\'")
                    .ignored()
                    .or(none_of(b'\'').ignored())
                    .repeated()
                    .to_slice(),
            )
            .then_ignore(just(b'\''))
            .map(ByteStr::new);
        let dqfstring = just(b"f\"")
            .ignore_then(
                just(b"\\\"")
                    .ignored()
                    .or(none_of(b'"').ignored())
                    .repeated()
                    .to_slice(),
            )
            .then_ignore(just(b'"'))
            .map(ByteStr::new);

        let comment =
            just(b'#').ignore_then(none_of(b'\n').repeated().to_slice().map(ByteStr::new));
        let term = one_of(b"\n;").repeated().at_least(1);
        let ws = one_of(b"\t ").repeated().at_least(1);

        let ident = any()
            .filter(|b: &u8| b.is_ascii_alphabetic() || *b == b'_')
            .then(
                any()
                    .filter(|b: &u8| b.is_ascii_alphanumeric() || matches!(*b, b'_' | b'-' | b'.'))
                    .repeated(),
            )
            .to_slice()
            .map(ByteStr::new);

        choice((
            just(b'(').map(|_| Self::LParen),
            just(b')').map(|_| Self::RParen),
            just(b'|').map(|_| Self::Pipe),
            sqstring.map(Self::String),
            dqstring.map(Self::String),
            sqfstring.map(Self::FString),
            dqfstring.map(Self::FString),
            comment.map(Self::Comment),
            term.map(|_| Self::Term),
            ws.map(|_| Self::Whitespace),
            ident.map(Self::Ident),
        ))
    }
}
