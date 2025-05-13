//! [KebabValue] impl.

use ::std::{borrow::Borrow, fmt::Display, ops::Deref};

use ::proc_macro2::{Literal, Span, TokenStream};
use ::quote::{IdentFragment, ToTokens};
use ::syn::{
    Ident, LitInt, LitStr,
    ext::IdentExt,
    parse::{Lookahead1, ParseStream, Parser},
};

use crate::util::kw_kind;

kw_kind!(
    /// A parsed output type (has span).
    Ty
    /// What kind of output tokens to produce.
    TyKind (Default) {
        /// Output an identifier.
        [default]
        Ident ident,
        /// Output a string literal.
        LitStr str,
        /// Output an integer literal.
        LitInt int,
    }
);

/// Wrapper to check for equality, including type for values.
/// To avoid breaking rules for [Borrow].
#[derive(Debug, Clone, Copy)]
pub struct ValueTyEq<'a>(&'a Value);

impl PartialEq<ValueTyEq<'_>> for ValueTyEq<'_> {
    #[inline]
    fn eq(&self, other: &ValueTyEq) -> bool {
        self.0.value == other.0.value && self.0.ty == other.0.ty
    }
}

impl PartialEq<ValueTyEq<'_>> for Value {
    #[inline]
    fn eq(&self, other: &ValueTyEq<'_>) -> bool {
        self.ty_eq() == *other
    }
}

impl PartialEq<Value> for ValueTyEq<'_> {
    #[inline]
    fn eq(&self, other: &Value) -> bool {
        *self == other.ty_eq()
    }
}

/// Value passed internally.
///
/// It implements [Borrow], [AsRef] and [Deref] to [str] but neither [PartialEq], [Eq] nor [Hash], as there
/// is a sensible borrow but no sensible equality check or hashing mathching said borrow. See
/// [Value::ty_eq] for equality checking.
#[derive(Debug, Clone, Default)]
pub struct Value {
    /// String representation of value.
    value: String,
    /// Any spans of value.
    span: Option<Span>,
    /// Requested type of value.
    ty: TyKind,
}

impl Value {
    /// Parse an instance if lookahead peek matches.
    ///
    /// # Errors
    /// If a valid value peeked by lookahead cannot be parsed.
    pub fn lookahead_parse(
        input: ParseStream,
        lookahead: &Lookahead1,
    ) -> ::syn::Result<Option<Self>> {
        Ok(Some(if lookahead.peek(Ident) {
            Self::from(&input.call(Ident::parse_any)?)
        } else if lookahead.peek(LitStr) {
            Self::from(&input.parse::<LitStr>()?)
        } else if lookahead.peek(LitInt) {
            Self::try_from(&input.parse::<LitInt>()?)?
        } else {
            return Ok(None);
        }))
    }

    /// Set  the type used for output.
    pub const fn set_ty(&mut self, ty: TyKind) -> &mut Self {
        self.ty = ty;
        self
    }

    /// Push a span to be used.
    pub fn push_span(&mut self, new_span: Span) -> &mut Self {
        if let Some(span) = self.span {
            if let span @ Some(..) = span.join(new_span) {
                self.span = span;
            }
        } else {
            self.span = Some(new_span)
        }
        self
    }

    /// Set the span.
    pub const fn set_span(&mut self, span: Span) -> &mut Self {
        self.span = Some(span);
        self
    }

    /// Get span of value.
    pub const fn span(&self) -> Option<Span> {
        self.span
    }

    /// Get an implementor of [PartialEq] that respects type.
    pub const fn ty_eq(&self) -> ValueTyEq {
        ValueTyEq(self)
    }

    /// Create a new ident value.
    pub fn new_ident(i: &str) -> Self {
        Self {
            value: i.into(),
            span: None,
            ty: TyKind::Ident,
        }
    }

    /// Create a new string literal value.
    pub fn new_litstr(i: &str) -> Self {
        Self {
            value: i.into(),
            span: None,
            ty: TyKind::LitStr,
        }
    }

    /// Create a new integer literal value.
    pub fn new_litint(i: &str) -> Self {
        Self {
            value: i.into(),
            span: None,
            ty: TyKind::LitInt,
        }
    }

    /// get value as a string slice.
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }

    /// Run a remapping function on value, keeping any spans and type.
    pub fn remap_value<M>(&mut self, mut m: M)
    where
        M: FnMut(String) -> String,
    {
        self.value = m(::std::mem::take(&mut self.value));
    }

    /// Convert into a [TypedValue].
    ///
    /// # Errors
    /// If the value and type are not compatible.
    pub fn try_to_typed(&self) -> ::syn::Result<TypedValue> {
        self.try_into()
    }

    /// Helper to join a set of values, keeping spans.
    pub fn join<J>(values: Vec<Self>, j: J) -> Self
    where
        J: FnOnce(Vec<Self>) -> String,
    {
        let mut value = Self::default();
        for v in &values {
            if let Some(span) = v.span() {
                value.push_span(span);
            }
        }
        value.value = j(values);

        value
    }

    /// Helper to split a set of values.
    pub fn split<S>(values: &[Self], mut s: S) -> Vec<Self>
    where
        S: for<'a> FnMut(&'a str) -> Vec<String>,
    {
        let mut out = Vec::new();
        for value in values {
            let ty = value.ty;
            let span = value.span;
            let values = s(value);
            let mapper = |value| Self { ty, span, value };

            out.reserve(values.len());
            out.extend(values.into_iter().map(mapper));
        }
        out
    }
}

impl Deref for Value {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Value {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for Value {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.value, f)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self {
            value,
            span: None,
            ty: TyKind::LitStr,
        }
    }
}

impl From<Value> for String {
    fn from(value: Value) -> Self {
        value.value
    }
}

impl TryFrom<&LitInt> for Value {
    type Error = ::syn::Error;

    fn try_from(value: &LitInt) -> Result<Self, Self::Error> {
        Ok(Self {
            value: value.base10_parse::<isize>()?.to_string(),
            span: Some(value.span()),
            ty: TyKind::LitInt,
        })
    }
}

impl From<&Ident> for Value {
    fn from(value: &Ident) -> Self {
        struct Wrap<'s>(&'s Ident);
        impl Display for Wrap<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                IdentFragment::fmt(self.0, f)
            }
        }
        Self {
            value: Wrap(value).to_string(),
            span: Some(value.span()),
            ty: TyKind::Ident,
        }
    }
}

impl From<&LitStr> for Value {
    fn from(value: &LitStr) -> Self {
        Self {
            value: value.value(),
            span: Some(value.span()),
            ty: TyKind::LitStr,
        }
    }
}

/// A typed [KebabValue] which may be converted to tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedValue {
    /// Value is an [identifier][Ident].
    Ident(Ident),
    /// Value is a [string literal][LitStr].
    LitStr(LitStr),
    /// Value i an [integer literal][LitInt]
    LitInt(LitInt),
}

impl TryFrom<&Value> for TypedValue {
    type Error = ::syn::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let span = value.span().unwrap_or_else(Span::call_site);
        Ok(match value.ty {
            TyKind::Ident => {
                let ident = Ident::parse_any
                    .parse_str(&value.value)
                    .map_err(|err| ::syn::Error::new(span, err))?;
                Self::Ident(ident)
            }
            TyKind::LitStr => Self::LitStr(::syn::LitStr::new(&value.value, span)),
            TyKind::LitInt => {
                let mut lit = Literal::isize_unsuffixed(
                    value
                        .value
                        .parse()
                        .map_err(|err| ::syn::Error::new(span, err))?,
                );
                lit.set_span(span);
                Self::LitInt(lit.into())
            }
        })
    }
}

impl TryFrom<Value> for TypedValue {
    type Error = ::syn::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.try_to_typed()
    }
}

impl ToTokens for TypedValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            TypedValue::Ident(ident) => ident.to_tokens(tokens),
            TypedValue::LitStr(lit_str) => lit_str.to_tokens(tokens),
            TypedValue::LitInt(lit_int) => lit_int.to_tokens(tokens),
        }
    }
}
