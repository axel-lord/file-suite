//! [DeferredArgs] impl.

use ::std::{fmt::Debug, marker::PhantomData};

use ::file_suite_proc_lib::{FromArg, ToArg};
use ::quote::ToTokens;
use ::syn::parse::Parse;

use crate::array_expr::function::{Call, ToCallable};

/// Deferred argument parse implementor.
pub struct DeferredArgs<C>
where
    C: FromArg,
{
    /// Parsed argument.
    arg: C::Factory,
    /// Allow for c to exist.
    _p: PhantomData<fn() -> C>,
}

impl<C> ToTokens for DeferredArgs<C>
where
    C: FromArg,
    C::Factory: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { arg, _p } = self;
        arg.to_tokens(tokens);
    }
}

impl<C> Parse for DeferredArgs<C>
where
    C: FromArg,
    C::Factory: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            arg: input.parse()?,
            _p: PhantomData,
        })
    }
}

impl<C> ToCallable for DeferredArgs<C>
where
    C: FromArg + Call,
{
    type Call = C;

    fn to_callable(&self) -> Self::Call {
        C::from_arg(self.arg.to_arg())
    }
}

impl<C> Debug for DeferredArgs<C>
where
    C: FromArg,
    C::Factory: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Args")
            .field("arg", &self.arg)
            .field("_p", &self._p)
            .finish()
    }
}

impl<C> Clone for DeferredArgs<C>
where
    C: FromArg,
    C::Factory: Clone,
{
    fn clone(&self) -> Self {
        Self {
            arg: self.arg.clone(),
            _p: PhantomData,
        }
    }
}
