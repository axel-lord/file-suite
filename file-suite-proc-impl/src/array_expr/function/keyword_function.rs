//! [KwFn] impl.

use ::file_suite_proc_lib::Lookahead;
use ::quote::ToTokens;
use ::syn::parse::Parse;
use syn::parse::{Lookahead1, ParseStream};

use crate::array_expr::function::ToCallable;

/// Function composed of a keyword K, and some arguments A.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KwFn<K, A> {
    /// Keyword identifying function.
    pub keyword: K,
    /// Function arguments.
    pub args: A,
}

impl<K, A> ToCallable for KwFn<K, A>
where
    A: ToCallable,
{
    type Call = A::Call;

    fn to_callable(&self) -> Self::Call {
        self.args.to_callable()
    }
}

impl<K, A> ToTokens for KwFn<K, A>
where
    K: ToTokens,
    A: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { keyword, args } = self;
        keyword.to_tokens(tokens);
        args.to_tokens(tokens);
    }
}

impl<K, A> Lookahead for KwFn<K, A>
where
    K: Lookahead,
{
    fn lookahead_peek(lookahead: &Lookahead1) -> bool {
        K::lookahead_peek(lookahead)
    }
}

impl<K, A> Parse for KwFn<K, A>
where
    K: Parse,
    A: Parse,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            keyword: input.parse()?,
            args: input.parse()?,
        })
    }
}
