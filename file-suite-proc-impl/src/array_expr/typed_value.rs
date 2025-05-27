//! [TypedValue] impl

use ::proc_macro2::{Literal, Span, TokenStream};
use ::quote::{ToTokens, TokenStreamExt};
use ::syn::{
    Ident, LitBool, LitInt, LitStr,
    ext::IdentExt,
    parse::{Lookahead1, ParseStream},
};

use crate::{array_expr::value::Value, util::lookahead_parse::LookaheadParse};

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
        }
    }
}

impl LookaheadParse for TypedValue {
    fn lookahead_parse(input: ParseStream, lookahead: &Lookahead1) -> syn::Result<Option<Self>> {
        Ok(Some(if lookahead.peek(Ident) {
            Self::Ident(input.call(Ident::parse_any)?)
        } else if lookahead.peek(LitStr) {
            Self::LitStr(input.parse()?)
        } else if lookahead.peek(LitInt) {
            let lit_int = input.parse::<LitInt>()?;
            Self::LitInt(lit_int.base10_parse()?, lit_int.span())
        } else {
            return Ok(None);
        }))
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
        }
    }
}
