//! [TypedValue] impl

use ::file_suite_proc_lib::{Lookahead, lookahead::ParseBufferExt};
use ::proc_macro2::{Literal, Span, TokenStream};
use ::quote::{ToTokens, TokenStreamExt};
use ::syn::{
    Ident, LitBool, LitInt, LitStr, Token,
    ext::IdentExt,
    parse::{Lookahead1, Parse, ParseStream},
};

use crate::value::Value;

/// A typed [Value] which may be converted to tokens.
#[derive(Debug, Clone)]
pub enum TypedValue {
    /// Value is an [identifier][Ident].
    Ident(Ident),
    /// Value is a [string literal][LitStr].
    LitStr(LitStr),
    /// Value is an [integer literal][LitInt].
    LitInt(isize, Span),
    /// Value is a [boolean literal][LitBool].
    LitBool(LitBool),
    /// Value is an [underscore][::syn::token::Underscore]
    Underscore(Token![_]),
    /// Value is a token stream (cannot be parsed, but is created in some contexts).
    Tokens(TokenStream),
}

impl TypedValue {
    /// Convert to a [Value]
    ///
    /// # Panics
    /// If of the Expr, Item or Stmt variants.
    pub fn to_value(&self) -> Value {
        match self {
            TypedValue::Ident(ident) => Value::from(ident),
            TypedValue::LitStr(lit_str) => Value::from(lit_str),
            TypedValue::LitBool(lit_bool) => Value::from(lit_bool),
            TypedValue::LitInt(value, span) => {
                let mut value = Value::new_int(*value);
                value.set_span(*span);
                value
            }
            TypedValue::Tokens(token_stream) => Value::new_tokens(token_stream.clone()),
            TypedValue::Underscore(underscore) => Value::new_tokens(underscore.to_token_stream()),
        }
    }
}

impl Lookahead for TypedValue {
    fn lookahead_peek(lookahead: &Lookahead1) -> bool {
        lookahead.peek(LitStr)
            || lookahead.peek(LitInt)
            || lookahead.peek(Ident)
            || lookahead.peek(Token![_])
    }

    fn lookahead_parse(input: ParseStream, lookahead: &Lookahead1) -> syn::Result<Option<Self>>
    where
        Self: Parse,
    {
        let value = if lookahead.peek(Ident) {
            Self::Ident(input.call(Ident::parse_any)?)
        } else if lookahead.peek(LitStr) {
            Self::LitStr(input.parse()?)
        } else if lookahead.peek(LitInt) {
            let lit_int = input.parse::<LitInt>()?;
            Self::LitInt(lit_int.base10_parse()?, lit_int.span())
        } else if lookahead.peek(Token![_]) {
            Self::Underscore(input.parse()?)
        } else {
            return Ok(None);
        };
        Ok(Some(value))
    }
}

impl Parse for TypedValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        match input.lookahead_parse(&lookahead)? {
            None => Err(lookahead.error()),
            Some(value) => Ok(value),
        }
    }
}

impl ToTokens for TypedValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            TypedValue::Ident(ident) => ident.to_tokens(tokens),
            TypedValue::LitStr(lit_str) => lit_str.to_tokens(tokens),
            TypedValue::LitInt(value, span) => {
                let mut literal = Literal::isize_unsuffixed(*value);
                literal.set_span(*span);
                tokens.append(literal);
            }
            TypedValue::LitBool(lit_bool) => lit_bool.to_tokens(tokens),
            TypedValue::Tokens(token_stream) => token_stream.to_tokens(tokens),
            TypedValue::Underscore(underscore) => underscore.to_tokens(tokens),
        }
    }
}
