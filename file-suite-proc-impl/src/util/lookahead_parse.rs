//! Utilities for parsing using a [Lookahead1].

use ::syn::parse::{Lookahead1, ParseStream};

/// Trait for conditional parsing useing a [Lookahead1].
pub trait LookaheadParse
where
    Self: Sized,
{
    /// Parse an instance if lookahead peek matches.
    ///
    /// # Errors
    /// If a valid value peeked by lookahead cannot be parsed.
    fn lookahead_parse(input: ParseStream, lookahead: &Lookahead1) -> ::syn::Result<Option<Self>>;

    /// Parse an instance if using [LookaheadParse::lookahead_parse] implementation.
    ///
    /// # Errors
    /// If an expected value cannot be parsed.
    /// Or if no expected value was encountered.
    fn parse(input: ParseStream) -> ::syn::Result<Self> {
        let lookahead = input.lookahead1();
        if let Some(value) = Self::lookahead_parse(input, &lookahead)? {
            Ok(value)
        } else {
            Err(lookahead.error())
        }
    }
}
