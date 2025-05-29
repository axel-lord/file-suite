//! Spanned integer value that validates on parse.

use ::std::num::NonZero;

use ::proc_macro2::{Span, TokenStream};
use ::quote::{ToTokens, quote_spanned};
use ::syn::{
    LitInt,
    parse::{Lookahead1, ParseStream},
};

use crate::{array_expr::function::ToArg, util::lookahead_parse::LookaheadParse};

#[doc(hidden)]
mod sealed {
    #[doc(hidden)]
    pub trait Sealed {}
}

/// Sealed trait for values which may be used with SpannedInt.
pub trait SpannedIntPrimitive
where
    Self: Sized + Copy + sealed::Sealed,
{
    #[doc(hidden)]
    fn from_lit(lit: LitInt) -> ::syn::Result<Self>;
    #[doc(hidden)]
    fn to_tokens(self, span: Span, tokens: &mut TokenStream);
}

impl_spanned_int_primitive! {
    usize, /* u128, u64, u32, u16, u8, */ isize, /* i128, i64, i32, i16, i8, */
}

/// An integer value with a span.
#[derive(Debug, Clone, Copy)]
pub struct SpannedInt<N>
where
    N: SpannedIntPrimitive,
{
    /// Integer value.
    pub value: N,
    /// Span of value.
    pub span: Span,
}

impl<N> ToArg for SpannedInt<N>
where
    N: SpannedIntPrimitive,
{
    type Arg = N;

    fn to_arg(&self) -> Self::Arg {
        self.value
    }
}

impl<N> ToTokens for SpannedInt<N>
where
    N: SpannedIntPrimitive,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { value, span } = *self;
        value.to_tokens(span, tokens);
    }
}

impl<N> LookaheadParse for SpannedInt<N>
where
    N: SpannedIntPrimitive,
{
    fn lookahead_parse(input: ParseStream, lookahead: &Lookahead1) -> syn::Result<Option<Self>> {
        lookahead
            .peek(LitInt)
            .then(|| {
                let lit_int = input.parse::<LitInt>()?;
                let span = lit_int.span();
                let value = N::from_lit(lit_int)?;

                Ok(Self { span, value })
            })
            .transpose()
    }
}

/// Implement trait for integer primitives and nonzero values.
macro_rules! impl_spanned_int_primitive {
    ($($ty:ty),* $(,)?) => {$(
        impl sealed::Sealed for $ty {}
        impl sealed::Sealed for NonZero<$ty> {}
        impl SpannedIntPrimitive for $ty {
            fn from_lit(lit: LitInt) -> syn::Result<Self> {
                lit.base10_parse()
            }

            fn to_tokens(self, span: Span, tokens: &mut TokenStream) {
                let value = self;
                tokens.extend(quote_spanned! {span=> #value});
            }
        }
        impl SpannedIntPrimitive for NonZero<$ty> {
            fn from_lit(lit: LitInt) -> syn::Result<Self> {
                lit.base10_parse()
            }

            fn to_tokens(self, span: Span, tokens: &mut TokenStream) {
                let value = self.get();
                tokens.extend(quote_spanned! {span=> #value});
            }
        }
    )*};
}
use impl_spanned_int_primitive;
