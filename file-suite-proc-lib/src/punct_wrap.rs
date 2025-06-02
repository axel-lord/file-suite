//! Wrappers for [Punctuated] implementing parse.

use ::quote::ToTokens;
use ::syn::{parse::Parse, punctuated::Punctuated};

use crate::Lookahead;

/// [Punctuated] wrapper implementing [Parse] using [Punctuated::parse_terminated].
#[derive(Debug, Clone)]
pub struct Terminated<T, P>(pub Punctuated<T, P>);

impl<T, P> Terminated<T, P> {
    /// Unwrap into the inner punctuated.
    pub fn into_inner(self) -> Punctuated<T, P> {
        let Self(p) = self;
        p
    }
}

impl<T, P> Parse for Terminated<T, P>
where
    T: Parse,
    P: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Punctuated::parse_terminated(input).map(Self)
    }
}

impl<T, P> Lookahead for Terminated<T, P>
where
    T: Lookahead,
{
    fn lookahead_peek(lookahead: &syn::parse::Lookahead1) -> bool {
        T::lookahead_peek(lookahead)
    }

    fn input_peek(input: syn::parse::ParseStream) -> bool {
        T::input_peek(input)
    }
}

impl<T, P> ToTokens for Terminated<T, P>
where
    T: ToTokens,
    P: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self(p) = self;
        p.to_tokens(tokens);
    }
}

/// [Punctuated] wrapper implementing [Parse] using [Punctuated::parse_separated_nonempty].
#[derive(Debug, Clone)]
pub struct Separated<T, P>(pub Punctuated<T, P>);

impl<T, P> Parse for Separated<T, P>
where
    T: Parse,
    P: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Punctuated::parse_terminated(input).map(Self)
    }
}

impl<T, P> Lookahead for Separated<T, P>
where
    T: Lookahead,
{
    fn lookahead_peek(lookahead: &syn::parse::Lookahead1) -> bool {
        T::lookahead_peek(lookahead)
    }

    fn input_peek(input: syn::parse::ParseStream) -> bool {
        T::input_peek(input)
    }
}

impl<T, P> ToTokens for Separated<T, P>
where
    T: ToTokens,
    P: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self(p) = self;
        p.to_tokens(tokens);
    }
}

impl<T, P> Separated<T, P> {
    /// Unwrap into the inner punctuated.
    pub fn into_inner(self) -> Punctuated<T, P> {
        let Self(p) = self;
        p
    }
}

