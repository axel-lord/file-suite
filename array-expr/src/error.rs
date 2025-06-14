//! [Error] impl.

use ::std::{fmt::Display, mem};

use ::file_suite_proc_lib::kw_kind::NotInSetError;
use ::proc_macro2::Span;

/// Crate error type, wraps and converts to [::syn::Error].
#[derive(Debug)]
pub enum Error {
    /// Error is an owned string.
    Owned(String),
    /// Error is a static string slice.
    Borrowed(&'static str),
    /// Error is a failed variable access.
    NoVar(String),
    /// Error is a syn error.
    Syn(::syn::Error),
}

impl Error {
    /// Convert inner value into a syn error, using the given span if needed.
    pub fn make_syn_error(&mut self, span: Span) -> &mut ::syn::Error {
        match self {
            Error::Owned(message) => *self = Self::Syn(::syn::Error::new(span, message)),
            Error::Borrowed(message) => *self = Self::Syn(::syn::Error::new(span, message)),
            Error::Syn(_) => (),
            Error::NoVar(key) => {
                *self = Self::Syn(::syn::Error::new(span, Self::NoVar(mem::take(key))))
            }
        }

        match self {
            Error::Syn(error) => error,
            _ => unreachable!(),
        }
    }

    /// Convert self into a [::syn::Error], using the given span if needed.
    pub fn into_syn_error(self, span: Span) -> ::syn::Error {
        match self {
            Error::Owned(message) => ::syn::Error::new(span, message),
            Error::Borrowed(message) => ::syn::Error::new(span, message),
            Error::Syn(err) => err,
            Error::NoVar(key) => ::syn::Error::new(span, Self::NoVar(key)),
        }
    }

    /// Convert any value implementing [Display] to self.
    pub fn from_display<E: Display>(err: E) -> Self {
        Self::Owned(err.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Owned(msg) => Display::fmt(msg, f),
            Error::Borrowed(msg) => Display::fmt(msg, f),
            Error::Syn(error) => Display::fmt(error, f),
            Error::NoVar(key) => write!(f, "could not find variable '{key}'"),
        }
    }
}

impl ::std::error::Error for Error {}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self::Borrowed(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Owned(value)
    }
}

impl From<::syn::Error> for Error {
    fn from(value: ::syn::Error) -> Self {
        Self::Syn(value)
    }
}

impl From<Error> for ::syn::Error {
    fn from(value: Error) -> Self {
        value.into_syn_error(Span::call_site())
    }
}

impl<Kw> From<NotInSetError<Kw>> for Error {
    fn from(value: NotInSetError<Kw>) -> Self {
        Self::Owned(value.to_string())
    }
}
