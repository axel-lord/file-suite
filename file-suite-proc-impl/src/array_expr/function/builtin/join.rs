//! [JoinArgs] impl.

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{LitChar, LitStr, parse::Parse};

use crate::{
    array_expr::{
        function::{Call, ToCallable},
        storage::Storage,
        value_array::ValueArray,
    },
    util::{kw_kind, lookahead_parse::lookahead_parse},
};

kw_kind!(
    /// Keyword specified join.
    JoinKw;
    /// Enum of possible values for [JoinKw].
    #[expect(non_camel_case_types)]
    JoinKind: Default {
        #[default]
        /// Concat values.
        concat,
        /// Join by dashes '-'.
        kebab,
        /// Join by underscores '_'.
        snake,
        /// Join by double colons '::'.
        path,
        /// Join by spaces ' '.
        space,
        /// Join by dots '.'.
        dot,
    }
);

/// Specification for how to join values.
#[derive(Debug, Clone)]
pub enum JoinArgs {
    /// Join by string.
    Str(LitStr),
    /// Join by char.
    Char(LitChar),
    /// Join according to keyword.
    Kw(JoinKw),
}

impl ToTokens for JoinArgs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Str(value) => value.to_tokens(tokens),
            Self::Char(value) => value.to_tokens(tokens),
            Self::Kw(value) => value.to_tokens(tokens),
        }
    }
}

impl Parse for JoinArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        Ok(if let Some(kw) = lookahead_parse(input, &lookahead)? {
            Self::Kw(kw)
        } else if let Some(s) = lookahead_parse(input, &lookahead)? {
            Self::Str(s)
        } else if let Some(chr) = lookahead_parse(input, &lookahead)? {
            Self::Char(chr)
        } else {
            return Err(lookahead.error());
        })
    }
}

/// [Call] implementor for [JoinArgs].
#[derive(Debug, Clone)]
pub enum JoinCallable {
    /// Join by a string.
    Str(String),
    /// Join by a char.
    Char(char),
    /// Join according to keyword.
    Kw(JoinKind),
}

impl Default for JoinCallable {
    fn default() -> Self {
        Self::Kw(JoinKind::concat)
    }
}

impl Call for JoinCallable {
    fn call(&self, input: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        Ok(match self {
            JoinCallable::Str(sep) => input.join_by_str(sep),
            JoinCallable::Char(sep) => {
                let mut buf = [0u8; 4];
                let sep = sep.encode_utf8(&mut buf) as &str;
                input.join_by_str(sep)
            }
            JoinCallable::Kw(kind) => match kind {
                JoinKind::concat => input.join_by_str(""),
                JoinKind::kebab => input.join_by_str("-"),
                JoinKind::snake => input.join_by_str("_"),
                JoinKind::path => input.join_by_str("::"),
                JoinKind::space => input.join_by_str(" "),
                JoinKind::dot => input.join_by_str("."),
            },
        })
    }
}

impl ToCallable for JoinArgs {
    type Call = JoinCallable;

    fn to_callable(&self) -> Self::Call {
        match self {
            JoinArgs::Str(lit_str) => JoinCallable::Str(lit_str.value()),
            JoinArgs::Char(lit_char) => JoinCallable::Char(lit_char.value()),
            JoinArgs::Kw(spec_kw) => JoinCallable::Kw(spec_kw.kind),
        }
    }
}
