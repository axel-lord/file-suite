//! [Value] impl.

use ::std::{borrow::Borrow, fmt::Display, ops::Deref};

use ::proc_macro2::Span;
use ::quote::IdentFragment;
use ::syn::{Ident, LitBool, LitInt, LitStr};

use crate::{
    array_expr::typed_value::TypedValue,
    util::{kw_kind, spanned_parse_str},
};

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
        self.0.content == other.0.content && self.0.ty == other.0.ty
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
    content: String,
    /// Any spans of value.
    span: Option<Span>,
    /// Requested type of value.
    ty: TyKind,
}

impl Value {
    /// Construct a new value with given content, and a type of [TyKind::str].
    pub const fn with_content(content: String) -> Self {
        Self {
            content,
            ty: TyKind::str,
            span: None,
        }
    }

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

    /// Set the string content of value.
    pub fn set_content(&mut self, content: String) -> &mut Self {
        self.content = content;
        self
    }

    /// Replace content of value, returning old content.
    #[inline]
    pub const fn replace_content(&mut self, content: String) -> String {
        ::std::mem::replace(&mut self.content, content)
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
            content: i,
            span: None,
            ty: TyKind::ident,
        }
    }

    /// Create a new string literal value.
    pub const fn new_str(i: String) -> Self {
        Self {
            content: i,
            span: None,
            ty: TyKind::str,
        }
    }

    /// Create a new integer literal value.
    pub fn new_int(i: isize) -> Self {
        Self {
            content: i.to_string(),
            span: None,
            ty: TyKind::int,
        }
    }

    /// get value as a string slice.
    pub const fn as_str(&self) -> &str {
        self.content.as_str()
    }

    /// Get value as a mutable string reference.
    #[expect(
        clippy::missing_const_for_fn,
        reason = "promise would be broken eventually"
    )]
    pub fn make_string(&mut self) -> &mut String {
        &mut self.content
    }

    /// Run a remapping function on value, keeping any spans and type.
    pub fn remap_value<M>(&mut self, m: M)
    where
        M: FnOnce(String) -> String,
    {
        self.content = m(::std::mem::take(&mut self.content));
    }

    /// Convert into a [TypedValue].
    ///
    /// # Errors
    /// If the value and type are not compatible.
    pub fn try_to_typed(&self) -> ::syn::Result<TypedValue> {
        let span = self.span.unwrap_or_else(Span::call_site);
        Ok(match self.ty {
            TyKind::ident => TypedValue::Ident(Ident::new(&self.content, span)),
            TyKind::str => TypedValue::LitStr(LitStr::new(&self.content, span)),
            TyKind::int => TypedValue::LitInt(
                self.parse().map_err(|err| ::syn::Error::new(span, err))?,
                span,
            ),
            TyKind::bool => TypedValue::LitBool(LitBool {
                value: self.parse().map_err(|err| ::syn::Error::new(span, err))?,
                span,
            }),
            TyKind::expr => TypedValue::Expr(Box::new(spanned_parse_str(span, self)?), span),
            TyKind::item => TypedValue::Item(Box::new(spanned_parse_str(span, self)?), span),
            TyKind::stmt => TypedValue::Stmt(Box::new(spanned_parse_str(span, self)?), span),
        })
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
        Display::fmt(&self.content, f)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self {
            content: value,
            span: None,
            ty: TyKind::str,
        }
    }
}

impl From<Value> for String {
    fn from(value: Value) -> Self {
        value.content
    }
}

impl TryFrom<&LitInt> for Value {
    type Error = ::syn::Error;

    fn try_from(value: &LitInt) -> Result<Self, Self::Error> {
        Ok(Self {
            content: value.base10_parse::<isize>()?.to_string(),
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
            content: Wrap(value).to_string(),
            span: Some(value.span()),
            ty: TyKind::ident,
        }
    }
}

impl From<&LitStr> for Value {
    fn from(value: &LitStr) -> Self {
        Self {
            content: value.value(),
            span: Some(value.span()),
            ty: TyKind::str,
        }
    }
}

impl From<&LitBool> for Value {
    fn from(value: &LitBool) -> Self {
        let LitBool { value, span } = value;

        Self {
            content: value.to_string(),
            span: Some(*span),
            ty: TyKind::bool,
        }
    }
}
