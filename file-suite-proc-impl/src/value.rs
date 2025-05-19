//! [Value] impl.

use ::std::{borrow::Borrow, fmt::Display, ops::Deref};

use ::proc_macro2::Span;
use ::quote::IdentFragment;
use ::syn::{Ident, LitBool, LitInt, LitStr};

use crate::{array_expr::value_array::ValueArray, typed_value::TypedValue, util::kw_kind};

kw_kind!(
    /// A parsed output type (has span).
    Ty;
    /// What kind of output tokens to produce.
    #[expect(non_camel_case_types)]
    TyKind: Default {
        /// Output an identifier.
        #[default]
        ident,
        /// Output a string literal.
        str,
        /// Output an integer literal.
        int,
        /// Output a boolean.
        bool,
        /// Output an expression.
        expr,
        /// Output an item.
        item,
        /// Value is a statement.
        stmt,
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
    /// Match type from other.
    pub fn with_ty_of(self, other: &Value) -> Self {
        Self {
            ty: other.ty,
            ..self
        }
    }

    /// Match span from other.
    pub fn with_span_of(self, other: &Value) -> Self {
        Self {
            span: other.span,
            ..self
        }
    }

    /// Set  the type used for output.
    pub const fn set_ty(&mut self, ty: TyKind) -> &mut Self {
        self.ty = ty;
        self
    }

    /// Get the type used for output.
    pub const fn ty(&self) -> TyKind {
        self.ty
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
    pub const fn new_ident(i: String) -> Self {
        Self {
            value: i,
            span: None,
            ty: TyKind::ident,
        }
    }

    /// Create a new string literal value.
    pub const fn new_str(i: String) -> Self {
        Self {
            value: i,
            span: None,
            ty: TyKind::str,
        }
    }

    /// Create a new integer literal value.
    pub fn new_int(i: isize) -> Self {
        Self {
            value: i.to_string(),
            span: None,
            ty: TyKind::int,
        }
    }

    /// get value as a string slice.
    pub const fn as_str(&self) -> &str {
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
    pub fn join<J>(values: ValueArray, j: J) -> Self
    where
        J: FnOnce(ValueArray) -> String,
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
    pub fn split<S>(values: &[Self], mut s: S) -> ValueArray
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
        out.into()
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
            ty: TyKind::str,
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
            ty: TyKind::int,
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
            ty: TyKind::ident,
        }
    }
}

impl From<&LitStr> for Value {
    fn from(value: &LitStr) -> Self {
        Self {
            value: value.value(),
            span: Some(value.span()),
            ty: TyKind::str,
        }
    }
}

impl From<&LitBool> for Value {
    fn from(value: &LitBool) -> Self {
        let LitBool { value, span } = value;

        Self {
            value: value.to_string(),
            span: Some(*span),
            ty: TyKind::bool,
        }
    }
}

impl TryFrom<&TypedValue> for Value {
    type Error = ::syn::Error;

    fn try_from(value: &TypedValue) -> Result<Self, Self::Error> {
        match value {
            TypedValue::Ident(ident) => Ok(Value::from(ident)),
            TypedValue::LitStr(lit_str) => Ok(Value::from(lit_str)),
            TypedValue::LitInt(lit_int) => Value::try_from(lit_int),
            TypedValue::LitBool(lit_bool) => Ok(Value::from(lit_bool)),
            TypedValue::Expr(_, span) | TypedValue::Item(_, span) | TypedValue::Stmt(_, span) => {
                Err(::syn::Error::new(
                    *span,
                    "Value should not be converted to from expr, stmt or item TypedValue",
                ))
            }
        }
    }
}

impl TryFrom<TypedValue> for Value {
    type Error = ::syn::Error;

    fn try_from(value: TypedValue) -> Result<Self, Self::Error> {
        value.try_to_value()
    }
}
