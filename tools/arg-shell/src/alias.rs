//! Trait "aliases"

use ::chumsky::{Parser, extra};

use crate::{token::Token, withspan::WithSpan};

/// Parser alias
pub trait TokenParser<'i, T>:
    Parser<
        'i,
        &'i [WithSpan<Token<'i>>],
        T,
        extra::Err<::chumsky::error::Rich<'i, WithSpan<Token<'i>>>>,
    >
{
}
impl<'i, T, V> TokenParser<'i, T> for V where
    V: Parser<
            'i,
            &'i [WithSpan<Token<'i>>],
            T,
            extra::Err<::chumsky::error::Rich<'i, WithSpan<Token<'i>>>>,
        >
{
}

/// Parser alias
pub trait ByteParser<'i, T>:
    Parser<'i, &'i [u8], T, extra::Err<::chumsky::error::Rich<'i, u8>>>
{
}
impl<'i, T, V> ByteParser<'i, T> for V where
    V: Parser<'i, &'i [u8], T, extra::Err<::chumsky::error::Rich<'i, u8>>>
{
}
