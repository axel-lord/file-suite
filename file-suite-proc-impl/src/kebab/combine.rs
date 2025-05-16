//! Implementation of combine part of expression.

use ::quote::ToTokens;
use ::syn::{
    LitChar, LitStr,
    parse::{Lookahead1, ParseStream},
};

use crate::{
    util::{kw_kind, lookahead_parse::LookaheadParse},
    value::{TyKind, Value},
};

/// Part of expression deciding how output should be combined.
#[derive(Debug, Clone)]
pub enum Combine {
    /// Combine by given value.
    LitStr(LitStr),
    /// Combine by given value.
    LitChar(LitChar),
    /// Decide by a single keyword.
    Keyword(CombineKeyword),
}

impl LookaheadParse for Combine {
    fn lookahead_parse(input: ParseStream, lookahead: &Lookahead1) -> syn::Result<Option<Self>> {
        Ok(Some(
            if let Some(value) = CombineKeyword::lookahead_parse(input, lookahead)? {
                Self::Keyword(value)
            } else if lookahead.peek(LitStr) {
                Self::LitStr(input.parse()?)
            } else if lookahead.peek(LitChar) {
                Self::LitChar(input.parse()?)
            } else {
                return Ok(None);
            },
        ))
    }
}

impl ToTokens for Combine {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Combine::Keyword(combine_keyword) => combine_keyword.to_tokens(tokens),
            Combine::LitStr(lit_str) => lit_str.to_tokens(tokens),
            Combine::LitChar(lit_char) => lit_char.to_tokens(tokens),
        }
    }
}

kw_kind!(
    /// Keyword for how output should be combined.
    CombineKeyword;
    /// How output should be combined.
    #[expect(non_camel_case_types)]
    CombineKeywordKind: Default {
        /// Values should be concatenated without any separator.
        #[default]
        concat,
        /// Values should be joined by a dash,
        kebab,
        /// Values should be joined by an underscore.
        snake,
        /// Values should be joined by a space.
        space,
        /// Values should be counted.
        count,
        /// Only the first value should be used.
        first,
        /// Only the last value should be used.
        last,
        /// Values should not be combined.
        split,
    }
);

impl CombineKeywordKind {
    /// Join input arguments.
    pub fn join(self, values: Vec<Value>) -> Vec<Value> {
        if matches!(self, CombineKeywordKind::split) {
            values
        } else {
            vec![Value::join(values, |values| match self {
                Self::concat => values.join(""),
                Self::kebab => values.join("-"),
                Self::snake => values.join("_"),
                Self::space => values.join(" "),
                Self::count => values.len().to_string(),
                Self::first => values
                    .into_iter()
                    .next()
                    .map(String::from)
                    .unwrap_or_default(),
                Self::last => values
                    .into_iter()
                    .next_back()
                    .map(String::from)
                    .unwrap_or_default(),
                Self::split => unreachable!(),
            })]
        }
    }

    /// Preferred [TyKind] of variant.
    pub const fn default_ty(self) -> Option<TyKind> {
        Some(match self {
            Self::count => TyKind::int,
            Self::space | Self::kebab => TyKind::int,
            _ => return None,
        })
    }
}
