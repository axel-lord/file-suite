//! MacroDelim extensions.

use ::proc_macro2::TokenStream;
use ::syn::{
    MacroDelimiter,
    parse::{Lookahead1, ParseStream},
    token::{Brace, Bracket, Paren},
};

/// Extension trait for [MacroDelimiter].
pub trait MacroDelimExt {
    /// Peek lookahead for any macro delimiter, brace, bracket, paren.
    fn lookahead_peek(lookahead: &Lookahead1) -> bool;

    /// Peek parse stream for any macro delimiter, same as [MacroDelimExt::lookahead_peek].
    fn input_peek(input: ParseStream) -> bool;

    /// Use delimiter to surround a value.
    fn surround<F>(&self, tokens: &mut TokenStream, f: F)
    where
        F: FnOnce(&mut TokenStream);
}

impl MacroDelimExt for MacroDelimiter {
    fn lookahead_peek(lookahead: &Lookahead1) -> bool {
        lookahead.peek(Paren) || lookahead.peek(Bracket) || lookahead.peek(Brace)
    }

    fn input_peek(input: ParseStream) -> bool {
        input.peek(Paren) || input.peek(Bracket) || input.peek(Brace)
    }

    fn surround<F>(&self, tokens: &mut TokenStream, f: F)
    where
        F: FnOnce(&mut TokenStream),
    {
        match self {
            MacroDelimiter::Paren(paren) => paren.surround(tokens, f),
            MacroDelimiter::Brace(brace) => brace.surround(tokens, f),
            MacroDelimiter::Bracket(bracket) => bracket.surround(tokens, f),
        }
    }
}

/// Parse a set of delimiters, brackets, braces or paren and expose their content.
#[macro_export]
macro_rules! macro_delimited {
    ($content:ident in $expr:expr) => {{
        let lookahead = $crate::__private::ParseBuffer::lookahead1($expr);
        if lookahead.peek($crate::__private::Bracket) {
            $crate::__private::MacroDelimiter::Bracket($crate::__private::bracketed!($content in $expr))
        } else if lookahead.peek($crate::__private::Brace) {
            $crate::__private::MacroDelimiter::Brace($crate::__private::braced!($content in $expr))
        } else if lookahead.peek($crate::__private::Paren) {
            $crate::__private::MacroDelimiter::Paren($crate::__private::parenthesized!($content in $expr))
        } else {
            return ::core::result::Result::Err($crate::__private::Lookahead1::error(lookahead));
        }
    }};
}
