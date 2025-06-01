//! [EmptyArgs] impl.

use ::std::marker::PhantomData;

use ::file_suite_proc_lib::{Lookahead, ensure_empty};
use ::quote::ToTokens;
use ::syn::{MacroDelimiter, parse::Parse};
use syn::parse::{Lookahead1, ParseStream};

use crate::{
    array_expr::function::{Call, DefaultArgs, ToCallable},
    macro_delimited,
    util::delimited::MacroDelimExt,
};

/// A delimited empty group, {}, [], (), which may not exist.
/// The given type T, is used to get a [Call] impl for [ToCallable].
#[derive(Debug, Clone)]
pub struct EmptyArgs<T> {
    /// Group delimiter.
    pub delim: Option<MacroDelimiter>,
    /// Allow for T to exist.
    _p: PhantomData<fn() -> T>,
}

impl<T> Parse for EmptyArgs<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if MacroDelimiter::input_peek(input) {
            let content;
            let delim = macro_delimited!(content in input);
            ensure_empty(&content)?;
            Self {
                delim: Some(delim),
                _p: PhantomData,
            }
        } else {
            Self {
                delim: None,
                _p: PhantomData,
            }
        })
    }
}

impl<T> Lookahead for EmptyArgs<T> {
    fn lookahead_peek(_: &Lookahead1) -> bool {
        true
    }
}

impl<T> ToCallable for EmptyArgs<T>
where
    T: DefaultArgs + Call,
{
    type Call = T;

    fn to_callable(&self) -> Self::Call {
        T::default_args()
    }
}

impl<T> ToTokens for EmptyArgs<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { delim, _p } = self;
        if let Some(delim) = delim {
            delim.surround(tokens, |_| {});
        }
    }
}
