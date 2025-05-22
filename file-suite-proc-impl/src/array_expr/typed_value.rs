//! [TypedValue] impl

use ::proc_macro2::{Literal, Span, TokenStream};
use ::quote::{ToTokens, TokenStreamExt, quote_spanned};
use ::syn::{
    Expr, Ident, Item, LitBool, LitInt, LitStr, Stmt,
    ext::IdentExt,
    parse::{Lookahead1, ParseStream},
};

use crate::{array_expr::value::Value, util::lookahead_parse::LookaheadParse};

/// A typed [KebabValue] which may be converted to tokens.
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
    /// Value is an expression (cannot be parsed).
    Expr(Box<Expr>, Span),
    /// Value is an item (cannot be parsed).
    Item(Box<Item>, Span),
    /// Value is a statement (cannot be parsed).
    Stmt(Box<Stmt>, Span),
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
            TypedValue::Expr(..) | TypedValue::Item(..) | TypedValue::Stmt(..) => {
                panic!("Value should not be converted to from expr, stmt or item TypedValue")
            }
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
            TypedValue::Expr(expr, span) => tokens.extend(quote_spanned! {*span=> #expr}),
            TypedValue::Item(item, span) => tokens.extend(quote_spanned! {*span=> #item}),
            TypedValue::Stmt(stmt, span) => tokens.extend(quote_spanned! {*span=> #stmt}),
        }
    }
}
