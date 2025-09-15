//! Abstract syntax tree implementation.

use ::std::borrow::Cow;

use ::chumsky::{Parser, select, span::SimpleSpan};

use crate::{
    ByteStr, alias::TokenParser, ast::arg::Arg, smallvec::SmallVec, token::Token,
    withspan::WithSpan,
};

pub mod arg;

pub mod new {
    //! new ast impl.

    use ::std::sync::OnceLock;

    use crate::{ByteStr, alias::ByteParser, withspan::WithSpan};

    /// Placeholder variant which may not exist.
    type PlaceHolderVariant<'i> = (
        ::core::convert::Infallible,
        ::core::marker::PhantomData<&'i ()>,
    );

    /// Ast expression.
    #[derive(Debug, Clone)]
    pub enum Expr<'i> {
        /// Unused variant.
        _P(PlaceHolderVariant<'i>),
    }

    /// Src expression.
    #[derive(Debug, Clone)]
    pub enum Src<'i> {
        /// Unused.
        _P(PlaceHolderVariant<'i>),
    }

    /// Sink expression.
    #[derive(Debug, Clone)]
    pub enum Sink<'i> {
        /// Unused.
        _P(PlaceHolderVariant<'i>),
    }

    /// An fstring argument.
    #[derive(Debug, Clone)]
    pub struct FString<'i> {
        content: WithSpan<&'i ByteStr>,
        cache: OnceLock<
            Result<crate::exec::fstring::FString<'i>, Vec<::chumsky::error::Rich<'i, u8>>>,
        >,
    }

    impl<'i> FString<'i> {
        /// Create a new instance from content.
        #[inline]
        pub const fn new(content: WithSpan<&'i ByteStr>) -> Self {
            Self {
                content,
                cache: OnceLock::new(),
            }
        }

        /// Get parsed fstring.
        pub fn parsed(
            &self,
            parser: &impl ByteParser<'i, crate::exec::fstring::FString<'i>>,
        ) -> Result<&'_ crate::exec::fstring::FString<'i>, &'_ [::chumsky::error::Rich<'i, u8>]>
        {
            self.cache
                .get_or_init(|| parser.parse(self.content.as_bytes()).into_result())
                .as_ref()
                .map_err(|err| err.as_slice())
        }
    }

    /// Ast argument.
    #[derive(Debug, Clone)]
    pub enum Arg<'i> {
        /// String argument.
        String(WithSpan<&'i ByteStr>),
        /// FString argument.
        FString(FString<'i>),
        /// Expression argument.
        Expr(Expr<'i>),
    }

    /// Ast command.
    #[derive(Debug, Clone)]
    pub struct Cmd<'i>(pub Vec<Arg<'i>>);

    /// Ast command chain.
    #[derive(Debug, Clone)]
    pub enum Chain<'i> {
        /// Src - Cmd... - Sink
        SrcCmdSink(Src<'i>, Vec<Cmd<'i>>, Sink<'i>),
        /// Cmd... - Sink
        CmdSink(Vec<Cmd<'i>>, Sink<'i>),
        /// Src - Sink
        SrcSink(Src<'i>, Sink<'i>),
    }
}

/// Variable scope.
#[derive(Debug, Clone, Copy)]
pub struct Variables<'a> {
    top: &'a [(Cow<'a, str>, Cow<'a, [u8]>)],
    next: Option<&'a Variables<'a>>,
}

impl<'a> Variables<'a> {
    /// Get a variable by name.
    pub fn get(&'a self, name: &str) -> Option<SmallVec<1, &'a ByteStr>> {
        let mut v = Some(self);

        while let Some(variables) = v.take() {
            for (key, value) in variables.top {
                if key.as_ref() == name {
                    return Some(SmallVec([ByteStr::new(value)].into()));
                }
            }
            v = variables.next;
        }

        None
    }
}

/// Command line call.
#[derive(Debug, Clone)]
pub struct Cmdline<'i>(pub Vec<Arg<'i>>);

/// Calls, builtins and commands.
#[derive(Debug, Clone)]
pub enum Call<'i> {
    /// Call a command line.
    Cmd(Cmdline<'i>),
    /// Pipe stdin.
    Stdin(SimpleSpan),
    /// Pipe to stdout.
    Stdout(SimpleSpan),
    /// Pipe to stderr.
    Stderr(SimpleSpan),
}

/// Create a parser to parse a specific keyword.
fn kw<'i>(kw: &'static str) -> impl TokenParser<'i, SimpleSpan> + Clone + Copy {
    select! {
        WithSpan { value: Token::Ident(kw_str), span } if kw_str == kw => span,
    }
    .labelled(kw)
}

/// Calls separated by pipes.
#[derive(Debug, Clone)]
pub struct Ast<'i>(pub Vec<Call<'i>>);

impl<'i> Ast<'i> {
    /// Get a parser for an ast.
    pub fn parser() -> impl TokenParser<'i, Self> + Clone {
        use ::chumsky::prelude::*;

        let skip = any()
            .filter(|token: &WithSpan<Token>| token.is_whitespace() || token.is_comment())
            .repeated();

        let ident = select! {
            WithSpan { value: Token::Ident(s), span } => WithSpan::from((ByteStr::new(s.as_bytes()), span)),
        };

        let string = select! {
            WithSpan { value: Token::String(byte_str), span } => WithSpan { value: byte_str, span },
        };

        let fstring = select! {
            WithSpan { value: Token::FString(byte_str), span } => WithSpan { value: byte_str, span },
        };

        let rparen = any().filter(|token: &WithSpan<Token>| token.is_r_paren());
        let lparen = any().filter(|token: &WithSpan<Token>| token.is_l_paren());

        recursive(|chain| {
            let group = chain.delimited_by(lparen, rparen);

            let arg = choice((
                string.map(Arg::String),
                ident.map(Arg::String),
                group.map(Arg::Group),
                fstring.map(crate::ast::arg::FString::new).map(Arg::FString),
            ))
            .padded_by(skip);

            let cmdline = arg.repeated().at_least(1).collect::<Vec<_>>().map(Cmdline);

            let padded_kw = |k: &'static str| kw(k).padded_by(skip);

            let call = choice((
                padded_kw("stdin").map(Call::Stdin),
                padded_kw("stdout").map(Call::Stdout),
                padded_kw("stderr").map(Call::Stderr),
                Parser::map(cmdline, Call::Cmd),
            ));

            // rust-analyzer (not cargo build) finds the wrong map function if
            // used in the normal way.
            Parser::map(
                call.separated_by(any().filter(|token: &WithSpan<Token>| token.is_pipe()))
                    .collect::<Vec<_>>(),
                Self,
            )
        })
    }
}
