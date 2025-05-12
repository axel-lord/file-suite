//! Macro for macro delimited content.

/// Extension trait for [MacroDelimiter].
pub trait MacroDelimExt {
    /// Peek lookahead for any macro delimiter, brace, bracket, paren.
    fn peek(lookahead: &Lookahead1) -> bool;

    /// Use delimiter to surround a value.
    fn surround<F>(&self, tokens: &mut TokenStream, f: F)
    where
        F: FnOnce(&mut TokenStream);
}

impl MacroDelimExt for MacroDelimiter {
    fn peek(lookahead: &Lookahead1) -> bool {
        lookahead.peek(Paren) || lookahead.peek(Bracket) || lookahead.peek(Brace)
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
macro_rules! macro_delimited {
    ($content:ident in $cursor:expr) => {{
        let lookahead = $cursor.lookahead1();
        if lookahead.peek(::syn::token::Bracket) {
            ::syn::MacroDelimiter::Bracket(::syn::bracketed! ( $content in $cursor ))
        } else if lookahead.peek(::syn::token::Brace) {
            ::syn::MacroDelimiter::Brace(::syn::braced! ( $content in $cursor ))
        } else if lookahead.peek(::syn::token::Paren) {
            ::syn::MacroDelimiter::Paren(::syn::parenthesized! ( $content in $cursor ))
        } else {
            return ::core::result::Result::Err(lookahead.error());
        }
    }};
}
use ::proc_macro2::TokenStream;
use ::syn::{
    MacroDelimiter,
    parse::Lookahead1,
    token::{Brace, Bracket, Paren},
};
pub(crate) use macro_delimited;
