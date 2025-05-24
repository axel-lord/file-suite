//! [Value] impl.

use ::std::{borrow::Borrow, cell::OnceCell, fmt::Display, ops::Deref};

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
        /// No type, cannot be converted to tokens.
        none,
    }
);

/// Wrapper to check for equality, including type for values.
/// To avoid breaking rules for [Borrow].
#[derive(Debug, Clone, Copy)]
pub struct ValueTyEq<'a>(&'a Value);

impl PartialEq<ValueTyEq<'_>> for ValueTyEq<'_> {
    #[inline]
    fn eq(&self, other: &ValueTyEq) -> bool {
        self.0.as_str() == other.0.as_str() && self.0.ty == other.0.ty
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

/// Content with cached string content.
#[derive(Debug, Clone)]
struct WithCache<T>(T, OnceCell<String>)
where
    T: ToString;

impl<T> WithCache<T>
where
    T: ToString,
{
    /// Create a new value with cache.
    const fn new(value: T) -> Self {
        Self(value, OnceCell::new())
    }

    /// Get value as a string slice.
    fn as_str(&self) -> &str {
        self.1.get_or_init(|| self.0.to_string())
    }
}

impl<T> From<WithCache<T>> for String
where
    T: ToString,
{
    fn from(value: WithCache<T>) -> Self {
        let WithCache(value, cache) = value;

        if let Some(value) = cache.into_inner() {
            return value;
        }

        value.to_string()
    }
}

/// Content of value.
#[derive(Debug, Clone)]
enum Content {
    /// Content is a string.
    String(String),
    /// Content is an integer.
    Int(WithCache<isize>),
    /// Content is a boolean.
    Bool(WithCache<bool>),
}

impl Default for Content {
    fn default() -> Self {
        Self::String(String::new())
    }
}

impl Content {
    /// Turn content into a string.
    fn make_string(&mut self) -> &mut String {
        *self = Self::String(match ::std::mem::take(self) {
            Content::String(value) => value,
            Content::Int(value) => String::from(value),
            Content::Bool(value) => String::from(value),
        });

        match self {
            Self::String(string) => string,
            _ => unreachable!(),
        }
    }

    /// Get content as a string slice.
    fn as_str(&self) -> &str {
        match self {
            Content::String(string) => string,
            Content::Int(value) => value.as_str(),
            Content::Bool(value) => value.as_str(),
        }
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
    content: Content,
    /// Any spans of value.
    span: Option<Span>,
    /// Requested type of value.
    pub ty: TyKind,
}

impl Value {
    /// Construct a new value with given content.
    pub const fn new(content: String) -> Self {
        Self {
            content: Content::String(content),
            ty: TyKind::str,
            span: None,
        }
    }

    /// Construct a new value from an integer.
    pub const fn new_int(value: isize) -> Self {
        Self {
            content: Content::Int(WithCache::new(value)),
            span: None,
            ty: TyKind::int,
        }
    }

    /// Get self with specified type.
    pub fn with_ty(self, ty: TyKind) -> Self {
        Self { ty, ..self }
    }

    /// Get self with specified content.
    pub fn with_content(self, content: String) -> Self {
        Self {
            content: Content::String(content),
            ..self
        }
    }

    /// Get self with specified span.
    pub fn with_span(self, span: Span) -> Self {
        Self {
            span: Some(span),
            ..self
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

    /// Get value as a mutable string reference.
    pub fn make_string(&mut self) -> &mut String {
        self.content.make_string()
    }

    /// get value as a string slice.
    pub fn as_str(&self) -> &str {
        self.content.as_str()
    }

    /// Get an implementor of [PartialEq] that respects type.
    pub const fn ty_eq(&self) -> ValueTyEq {
        ValueTyEq(self)
    }

    /// Convert into a [TypedValue].
    ///
    /// # Errors
    /// If the value and type are not compatible.
    pub fn try_to_typed(&self) -> ::syn::Result<TypedValue> {
        let span = self.span.unwrap_or_else(Span::call_site);
        Ok(match self.ty {
            TyKind::ident => TypedValue::Ident(Ident::new(self.as_str(), span)),
            TyKind::str => TypedValue::LitStr(LitStr::new(self.as_str(), span)),
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
            TyKind::none => {
                return Err(::syn::Error::new(
                    span,
                    "values of type none cannot be output",
                ));
            }
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
        Display::fmt(self.as_str(), f)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self {
            content: Content::String(value),
            span: None,
            ty: TyKind::str,
        }
    }
}

impl From<Value> for String {
    fn from(mut value: Value) -> Self {
        ::std::mem::take(value.make_string())
    }
}

impl TryFrom<&LitInt> for Value {
    type Error = ::syn::Error;

    fn try_from(value: &LitInt) -> Result<Self, Self::Error> {
        Ok(Self::new_int(value.base10_parse()?).with_span(value.span()))
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
        Self::new(Wrap(value).to_string())
            .with_span(value.span())
            .with_ty(TyKind::ident)
    }
}

impl From<&LitStr> for Value {
    fn from(value: &LitStr) -> Self {
        Self::new(value.value())
            .with_span(value.span())
            .with_ty(TyKind::str)
    }
}

impl From<&LitBool> for Value {
    fn from(value: &LitBool) -> Self {
        let LitBool { value, span } = value;

        Self {
            content: Content::Bool(WithCache::new(*value)),
            span: Some(*span),
            ty: TyKind::bool,
        }
    }
}
