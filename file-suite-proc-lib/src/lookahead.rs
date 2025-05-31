//! Utilities for lookahead parsing.

use ::syn::parse::{Lookahead1, Parse, ParseBuffer, ParseStream};

/// Trait for types which should be parsed based on the next token in a lookahead.
pub trait Lookahead {
    /// Check if the next token indicates trait implementor should be parsed.
    fn lookahead_peek(lookahead: &Lookahead1) -> bool;

    /// Check if the next token in input indicates trait implementor should be parsed.
    #[inline]
    fn input_peek(input: ParseStream) -> bool {
        Self::lookahead_peek(&input.lookahead1())
    }
}

/// Extension trait for [ParseBuffer] using [Lookahead].
pub trait ParseBufferExt {
    /// Parse the type T if [Lookahead::lookahead_peek] returns true.
    ///
    /// # Errors
    /// If [Lookahead::lookahead_peek] returns true and then the parsing fails
    /// said error will be forwarded.
    fn lookahead_parse<T>(&self, lookahead: &Lookahead1) -> ::syn::Result<Option<T>>
    where
        T: Lookahead + Parse;

    /// Parse the type T if [Lookahead::input_peek] returns true.
    /// # Errors
    /// If [Lookahead::input_peek] returns true and then the parsing fails
    /// said error will be forwarded.
    fn optional_parse<T>(&self) -> ::syn::Result<Option<T>>
    where
        T: Lookahead + Parse;

    /// Parse the type T if [Lookahead::lookahead_peek] returns true.
    ///
    /// Then replace the [Lookahead1], if parse was successfull.
    ///
    /// # Errors
    /// If [Lookahead::lookahead_peek] returns true and then the parsing fails
    /// said error will be forwarded.
    fn forward_parse<'s, T>(&'s self, lookahead: &mut Lookahead1<'s>) -> ::syn::Result<Option<T>>
    where
        T: Lookahead + Parse;
}

impl ParseBufferExt for ParseBuffer<'_> {
    #[inline]
    fn lookahead_parse<T>(&self, lookahead: &Lookahead1) -> syn::Result<Option<T>>
    where
        T: Lookahead + Parse,
    {
        if T::lookahead_peek(lookahead) {
            self.parse().map(Some)
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn optional_parse<T>(&self) -> syn::Result<Option<T>>
    where
        T: Lookahead + Parse,
    {
        self.lookahead_parse(&self.lookahead1())
    }

    #[inline]
    fn forward_parse<'s, T>(&'s self, lookahead: &mut Lookahead1<'s>) -> syn::Result<Option<T>>
    where
        T: Lookahead + Parse,
    {
        match self.lookahead_parse(lookahead) {
            result @ (Err(..) | Ok(None)) => result,
            result @ Ok(Some(..)) => {
                *lookahead = self.lookahead1();
                result
            }
        }
    }
}
