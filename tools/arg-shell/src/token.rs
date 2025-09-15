//! Tokens used by parser.

use ::std::iter::once;

use ::chumsky::{error::RichPattern, label::LabelError, util::MaybeRef};

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
    fn filter(b: &u8) -> bool {
        matches!(*b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'=')
    }
    any()
        .filter(filter)
        .repeated()
        .at_least(1)
        .to_slice()
        .map(str::from_utf8)
        .unwrapped()
}

/// Parser parsing a raw string, NOT including leading r.
pub fn rstring<'i>() -> impl ByteParser<'i, &'i ByteStr> + Clone + Copy {
    use ::chumsky::prelude::*;
    custom(|inp| {
        let mut leading = b"".as_slice();
        let start = inp.cursor();
        let delim;
        loop {
            let before = inp.cursor();
            match inp.next() {
                Some(b'#') => {
                    leading = inp.slice_since((&start)..);
                }
                Some(d @ (b'"' | b'\'')) => {
                    delim = d;
                    break;
                }
                other => {
                    return Err(
                        <Rich<u8> as LabelError<&[u8], RichPattern<u8>>>::expected_found(
                            [b'"', b'\'', b'#']
                                .map(MaybeRef::Val)
                                .map(RichPattern::Token),
                            other.map(From::from),
                            inp.span_since(&before),
                        ),
                    );
                }
            }
        }

        loop {
            let before = inp.cursor();
            match inp.next() {
                Some(d) if d == delim => {
                    
                }
                None => {
                    return Err(
                        <Rich<u8> as LabelError<&[u8], RichPattern<u8>>>::expected_found(
                            once(RichPattern::Token(MaybeRef::Val(delim))),
                            None,
                            inp.span_since(&before),
                        ),
                    );
                }
                _ => {}
            }
        }

        println!("{}", delim);

        Ok(ByteStr::new(b""))
    })
}

impl<'i> Token<'i> {
    /// Get a token parser.
    pub fn parser() -> impl ByteParser<'i, Self> {
        use ::chumsky::prelude::*;

        let qstr = |delim: u8| {
            none_of([delim])
                .repeated()
                .to_slice()
                .delimited_by(just(delim), just(delim))
                .map(ByteStr::new)
        };

        let sqstring = qstr(b'\'');
        let dqstring = qstr(b'"');

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
            just(b'(').to(Self::LParen).labelled("Left Parentheses"),
            just(b')').to(Self::RParen).labelled("Right Parentheses"),
            just(b'|').to(Self::Pipe).labelled("Pipe"),
            sqstring.map(Self::String).labelled("Single Quoted String"),
            dqstring.map(Self::String).labelled("Double Quoted String"),
            sqfstring
                .map(Self::FString)
                .labelled("Single Quoted Format String"),
            dqfstring
                .map(Self::FString)
                .labelled("Double Quoted Format String"),
            comment.map(Self::Comment).labelled("Comment"),
            ws.to(Self::Whitespace).labelled("Whitespace"),
            ident().map(Self::Ident).labelled("Identifier"),
        ))
    }
}
