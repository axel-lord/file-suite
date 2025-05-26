//! [KwFn] impl.

use ::quote::ToTokens;
use syn::parse::{Lookahead1, ParseStream};

use crate::{
    array_expr::function::ToCallable,
    util::lookahead_parse::{LookaheadParse, lookahead_parse},
};

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

impl<K, A> LookaheadParse for KwFn<K, Option<A>>
where
    K: LookaheadParse,
    A: LookaheadParse,
{
    fn lookahead_parse(input: ParseStream, lookahead: &Lookahead1) -> syn::Result<Option<Self>> {
        lookahead_parse::<K>(input, lookahead)?
            .map(|keyword| {
                Ok(Self {
                    keyword,
                    args: A::optional_parse(input)?,
                })
            })
            .transpose()
    }
}

impl<K, A> LookaheadParse for KwFn<K, A>
where
    K: LookaheadParse,
    A: LookaheadParse,
{
    fn lookahead_parse(input: ParseStream, lookahead: &Lookahead1) -> syn::Result<Option<Self>> {
        lookahead_parse::<K>(input, lookahead)?
            .map(|keyword| {
                Ok(Self {
                    keyword,
                    args: A::parse(input)?,
                })
            })
            .transpose()
    }
}
