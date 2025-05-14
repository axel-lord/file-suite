//! [TypedValue] impl

use ::proc_macro2::{Literal, Span, TokenStream};
use ::quote::ToTokens;
use ::syn::{
    Ident, LitBool, LitInt, LitStr,
    ext::IdentExt,
    parse::{Lookahead1, Parse, ParseStream, Parser},
};

use crate::value::{TyKind, Value};

/// A typed [KebabValue] which may be converted to tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedValue {
    /// Value is an [identifier][Ident].
    Ident(Ident),
    /// Value is a [string literal][LitStr].
    LitStr(LitStr),
    /// Value is an [integer literal][LitInt].
    LitInt(LitInt),
    /// Value is a [boolean literal][litBool].
    LitBool(LitBool),
}

impl TypedValue {
    /// Parse an instance if lookahead peek matches.
    ///
    /// # Errors
    /// If a valid value peeked by lookahead cannot be parsed.
    pub fn lookahead_parse(
        input: ParseStream,
        lookahead: &Lookahead1,
    ) -> ::syn::Result<Option<Self>> {
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

    /// Try to convert into a regular [Value].
    ///
    /// # Errors
    /// If any of the types are incompatible with [Value], such as [LitInt] not being base10.
    pub fn try_to_value(&self) -> ::syn::Result<Value> {
        self.try_into()
    }
}

impl Parse for TypedValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if let Some(value) = Self::lookahead_parse(input, &lookahead)? {
            Ok(value)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for TypedValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            TypedValue::Ident(ident) => ident.to_tokens(tokens),
            TypedValue::LitStr(lit_str) => lit_str.to_tokens(tokens),
            TypedValue::LitInt(lit_int) => lit_int.to_tokens(tokens),
            TypedValue::LitBool(lit_bool) => lit_bool.to_tokens(tokens),
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
        })
    }
}

impl TryFrom<Value> for TypedValue {
    type Error = ::syn::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.try_to_typed()
    }
}
