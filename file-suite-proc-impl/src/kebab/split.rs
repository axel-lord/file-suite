//! Implementation of splitting part of expression.

use ::proc_macro2::{Literal, Span, TokenStream};
use ::quote::ToTokens;
use ::syn::{
    LitChar, LitInt, LitStr,
    parse::{Parse, ParseStream},
};

use crate::{util::kw_kind, value::Value};

/// Convert to string and collect to vec.
fn collect_strings<I, S>(i: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    String: From<S>,
{
    i.into_iter().map(String::from).collect()
}

/// Split value of input.
#[derive(Debug)]
pub enum Split {
    /// Split by given pattern.
    StrPattern(LitStr),
    /// Split by given pattern.
    CharPattern(LitChar),
    /// Split at index.
    AtIndex(isize, Span),
    /// Split according to keyword.
    Keyword(SplitKeyword),
}

impl Split {
    /// Transform args given to input into desired form.
    pub fn transform_args(split: Option<&Self>, args: &[Value]) -> Vec<Value> {
        let Some(split) = split else {
            return SplitKeywordKind::default().transform_args(args);
        };

        match split {
            Split::StrPattern(lit_str) => {
                let pat = &lit_str.value();
                Value::split(args, |s| collect_strings(s.split(pat)))
            }
            Split::CharPattern(lit_char) => {
                let pat = lit_char.value();
                Value::split(args, |s| collect_strings(s.split(pat)))
            }
            Split::AtIndex(idx, _) => {
                let idx = *idx;
                if idx < 0 {
                    Value::split(args, |s| {
                        collect_strings({
                            let idx = s.len() as isize + idx;

                            <[&str; 2]>::from(if idx < 0 {
                                ("", s)
                            } else {
                                let idx = idx as usize;

                                s.split_at_checked(idx).unwrap_or(("", s))
                            })
                        })
                    })
                } else {
                    let idx = idx as usize;
                    Value::split(args, |s| {
                        collect_strings(<[&str; 2]>::from(
                            s.split_at_checked(idx).unwrap_or((s, "")),
                        ))
                    })
                }
            }
            Split::Keyword(split_keyword) => split_keyword.transform_args(args),
        }
    }
}

impl ToTokens for Split {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Split::StrPattern(lit_str) => lit_str.to_tokens(tokens),
            Split::CharPattern(lit_char) => lit_char.to_tokens(tokens),
            Split::Keyword(split_keyword) => split_keyword.to_tokens(tokens),
            Split::AtIndex(i, span) => {
                let mut i = Literal::isize_unsuffixed(*i);
                i.set_span(*span);
                i.to_tokens(tokens);
            }
        }
    }
}

impl Parse for Split {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        Ok(if lookahead.peek(LitStr) {
            Self::StrPattern(input.parse()?)
        } else if lookahead.peek(LitInt) {
            let i: LitInt = input.parse()?;
            Self::AtIndex(i.base10_parse()?, i.span())
        } else if lookahead.peek(LitChar) {
            Self::CharPattern(input.parse()?)
        } else if let Some(value) = SplitKeyword::lookahead_parse(input, &lookahead)? {
            Self::Keyword(value)
        } else {
            return Err(lookahead.error());
        })
    }
}

kw_kind!(
    /// A Split that was parsed an as such has a span.
    SplitKeyword
    /// How input values should be split
    [expect(non_camel_case_types)]
    SplitKeywordKind (Default) {
        /// Values should be split as they are given.
        [default]
        split,
        /// Values should be split by camelCase or PascalCase convention.
        pascal,
        /// Values should be split by camelCase convention.
        camel,
        /// Values should be split by dashes.
        kebab,
        /// Values should be split by underscores.
        snake,
        /// Values should be split by spaces, ' '.
        space,
        /// Values should be split by double colons, '::'.
        path,
        /// Values should be split by dots, '.'.
        dot,
    }
);

impl SplitKeywordKind {
    /// Transform args given to input into desired form.
    pub fn transform_args(self, args: &[Value]) -> Vec<Value> {
        match self {
            Self::split => Vec::from(args),
            Self::pascal => Value::split(args, |s| {
                collect_strings(s.split(char::is_uppercase).skip(
                    if s.starts_with(char::is_uppercase) {
                        1
                    } else {
                        0
                    },
                ))
            }),
            Self::camel => Value::split(args, |s| collect_strings(s.split(char::is_uppercase))),
            Self::kebab => Value::split(args, |s| collect_strings(s.split('-'))),
            Self::snake => Value::split(args, |s| collect_strings(s.split('_'))),
            Self::space => Value::split(args, |s| collect_strings(s.split(' '))),
            Self::path => Value::split(args, |s| collect_strings(s.split("::"))),
            Self::dot => Value::split(args, |s| collect_strings(s.split('.'))),
        }
    }
}
