use crate::{ByteStr, alias::ByteParser};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ::derive_more::IsVariant)]
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
    Ident(&'i str),
    /// Tabs, line breaks and spaces.
    Whitespace,
}

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

        choice((
            just(b'(').map(|_| Self::LParen),
            just(b')').map(|_| Self::RParen),
            just(b'|').map(|_| Self::Pipe),
            sqstring.map(Self::String),
            dqstring.map(Self::String),
            sqfstring.map(Self::FString),
            dqfstring.map(Self::FString),
            comment.map(Self::Comment),
            ws.map(|_| Self::Whitespace),
            ident().map(Self::Ident),
        ))
    }
}
