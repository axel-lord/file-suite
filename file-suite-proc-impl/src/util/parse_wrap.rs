//! Implementation of wrapper wrapping a [LookaheadParse] implementor
//! to implement [Parse].

use ::std::ops::{Deref, DerefMut};

use ::quote::ToTokens;
use ::syn::parse::{Parse, ParseStream};

/// Wrap a [LookaheadParse] implementor to [Parse].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ParseWrap<T> {
    /// Wrapped value implementing [LookaheadParse].
    pub inner: T,
}

impl<T> DerefMut for ParseWrap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T> Deref for ParseWrap<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> AsMut<T> for ParseWrap<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> AsRef<T> for ParseWrap<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> ToTokens for ParseWrap<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { inner } = self;
        inner.to_tokens(tokens);
    }
}

impl<T> Parse for ParseWrap<T>
where
    T: Parse,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.call(T::parse).map(|inner| Self { inner })
    }
}
