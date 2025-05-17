//! [TypedValue] impl

use ::proc_macro2::{Literal, Span, TokenStream};
use ::quote::{ToTokens, quote_spanned};
use ::syn::{
    Expr, Ident, Item, LitBool, LitInt, LitStr, Stmt,
    ext::IdentExt,
    parse::{Lookahead1, Parse, ParseStream, Parser},
};

use crate::{
    util::lookahead_parse::LookaheadParse,
    value::{TyKind, Value},
};

/// A typed [KebabValue] which may be converted to tokens.
#[derive(Debug, Clone)]
pub enum TypedValue {
    /// Value is an [identifier][Ident].
    Ident(Ident),
    /// Value is a [string literal][LitStr].
    LitStr(LitStr),
    /// Value is an [integer literal][LitInt].
    LitInt(LitInt),
    /// Value is a [boolean literal][litBool].
    LitBool(LitBool),
    /// Value is an expression (cannot be parsed).
    Expr(Box<Expr>, Span),
    /// Value is an item (cannot be parsed).
    Item(Box<Item>, Span),
    /// Value is a statement (cannot be parsed).
    Stmt(Box<Stmt>, Span),
}

impl TypedValue {
    /// Try to convert into a regular [Value].
    ///
    /// # Errors
    /// If any of the types are incompatible with [Value], such as [LitInt] not being base10.
    pub fn try_to_value(&self) -> ::syn::Result<Value> {
        self.try_into()
    }
}

impl LookaheadParse for TypedValue {
    fn lookahead_parse(input: ParseStream, lookahead: &Lookahead1) -> syn::Result<Option<Self>> {
        Ok(Some(if lookahead.peek(Ident) {
            Self::Ident(input.call(Ident::parse_any)?)
        } else if lookahead.peek(LitStr) {
            Self::LitStr(input.parse()?)
        } else if lookahead.peek(LitInt) {
            Self::LitInt(input.parse()?)
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
            TypedValue::LitInt(lit_int) => lit_int.to_tokens(tokens),
            TypedValue::LitBool(lit_bool) => lit_bool.to_tokens(tokens),
            TypedValue::Expr(expr, span) => tokens.extend(quote_spanned! {*span=> #expr}),
            TypedValue::Item(item, span) => tokens.extend(quote_spanned! {*span=> #item}),
            TypedValue::Stmt(stmt, span) => tokens.extend(quote_spanned! {*span=> #stmt}),
        }
    }
}

impl TryFrom<&Value> for TypedValue {
    type Error = ::syn::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let span = value.span().unwrap_or_else(Span::call_site);
        Ok(match value.ty() {
            TyKind::ident => {
                let ident = Ident::parse_any
                    .parse_str(value.as_str())
                    .map_err(|err| ::syn::Error::new(span, err))?;
                Self::Ident(ident)
            }
            TyKind::str => Self::LitStr(::syn::LitStr::new(value.as_str(), span)),
            TyKind::int => {
                let mut lit = Literal::isize_unsuffixed(
                    value
                        .as_str()
                        .parse()
                        .map_err(|err| ::syn::Error::new(span, err))?,
                );
                lit.set_span(span);
                Self::LitInt(lit.into())
            }
            TyKind::bool => Self::LitBool(LitBool::new(
                value
                    .as_str()
                    .parse()
                    .map_err(|err| ::syn::Error::new(span, err))?,
                span,
            )),
            TyKind::expr => Self::Expr(
                Box::new(Expr::parse.parse_str(value.as_str())?),
                value.span().unwrap_or(Span::call_site()),
            ),
            TyKind::item => Self::Item(
                Box::new(Item::parse.parse_str(value.as_str())?),
                value.span().unwrap_or(Span::call_site()),
            ),
            TyKind::stmt => Self::Stmt(
                Box::new(Stmt::parse.parse_str(value.as_str())?),
                value.span().unwrap_or(Span::call_site()),
            ),
        })
    }
}

impl TryFrom<Value> for TypedValue {
    type Error = ::syn::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.try_to_typed()
    }
}
