use ::chumsky::{IterParser, Parser};

use crate::{
    ByteStr,
    alias::{ByteParser, TokenParser},
    smallvec::SmallVec,
    token::{Token, ident},
    withspan::WithSpan,
};

/// A Fragment of an fstring.
#[derive(Debug, Clone, Copy)]
pub enum FStringFragment<'i> {
    Text(&'i ByteStr),
    Lookup(&'i str),
}

impl<'i> FStringFragment<'i> {
    pub fn parser() -> impl ByteParser<'i, Self> + Clone + Copy {
        use ::chumsky::prelude::*;
        use FStringFragment::{Lookup, Text};

        choice((
            just(b"{{").to(Text(ByteStr::new(b"{"))),
            just(b"}}").to(Text(ByteStr::new(b"}"))),
            just(br"\\").to(Text(ByteStr::new(br"\"))),
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
pub struct FString<'i>(SmallVec<3, FStringFragment<'i>>);

impl<'i> FString<'i> {
    pub fn parser() -> impl ByteParser<'i, Self> + Clone + Copy {
        FStringFragment::parser().repeated().collect().map(Self)
    }
}

/// Arguments for calls.
#[derive(Debug, Clone)]
pub enum Arg<'i> {
    String(WithSpan<&'i ByteStr>),
    FString(FString<'i>),
    Group(Ast<'i>),
}

/// Command line call.
#[derive(Debug, Clone)]
pub struct Cmdline<'i>(Vec<Arg<'i>>);

/// Calls, builtins and commands.
#[derive(Debug, Clone)]
pub enum Call<'i> {
    Cmd(Cmdline<'i>),
}

/// Calls separated by pipes.
#[derive(Debug, Clone)]
pub struct Ast<'i>(Vec<Call<'i>>);

impl<'i> Ast<'i> {
    pub fn parser() -> impl TokenParser<'i, Self> + Clone {
        use ::chumsky::prelude::*;

        let skip = any()
            .filter(|token: &WithSpan<Token>| token.is_whitespace() || token.is_comment())
            .repeated();

        let string = select! {
            WithSpan { value: Token::String(byte_str), span } => WithSpan::from((byte_str, span)),
            WithSpan { value: Token::Ident(s), span } => WithSpan::from((ByteStr::new(s.as_bytes()), span)),
        };

        let fstring_parser = FString::parser();

        let fstring = select! {
            WithSpan { value: Token::FString(byte_str), span:_ } => byte_str
        }
        .try_map(move |byte_str, span| {
            fstring_parser
                .parse(byte_str.as_bytes())
                .into_result()
                .map_err(|err| {
                    use ::std::fmt::Write;
                    let mut buf = String::new();

                    for err in err {
                        writeln!(buf, "{err}").expect("write to string should succeed");
                    }

                    Rich::custom(span, buf.trim_end())
                })
        });

        let rparen = any().filter(|token: &WithSpan<Token>| token.is_r_paren());
        let lparen = any().filter(|token: &WithSpan<Token>| token.is_l_paren());

        recursive(|chain| {
            let group = chain.delimited_by(lparen, rparen);

            let arg = choice((
                string.map(Arg::String),
                group.map(Arg::Group),
                fstring.map(Arg::FString),
            ))
            .padded_by(skip);

            let cmdline = arg.repeated().at_least(1).collect::<Vec<_>>().map(Cmdline);

            let call = cmdline.map(Call::Cmd);

            call.separated_by(any().filter(|token: &WithSpan<Token>| token.is_pipe()))
                .collect::<Vec<_>>()
                .map(Self)
        })
    }
}
