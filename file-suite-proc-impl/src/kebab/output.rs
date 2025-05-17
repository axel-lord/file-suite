//! [Combine], [Case] and [Ty] impls.

use ::quote::ToTokens;
use ::syn::{
    MacroDelimiter,
    parse::{End, Parse},
};

use crate::{
    kebab::{
        case::{Case, CaseKind},
        combine::{CombineKeyword, CombineKeywordKind},
    },
    util::{MacroDelimExt, do_n_times_then, lookahead_parse::LookaheadParse, macro_delimited},
    value::{Ty, TyKind},
};

/// Kebab expression output.
#[derive(Debug, Default)]
pub struct KebabOutput {
    /// Ouput type.
    pub ty: Option<Ty>,
    /// How to combine and case arguments.
    pub combine_case_group: Option<CombineCaseGroup>,
}

impl KebabOutput {
    /// Get parsed [CombineKind] if any.
    pub fn combine(&self) -> Option<CombineKeywordKind> {
        self.combine_case_group
            .as_ref()
            .and_then(|g| g.combine)
            .as_deref()
            .copied()
    }

    /// Get parsed [CaseKind] if any.
    pub fn case(&self) -> Option<CaseKind> {
        self.combine_case_group
            .as_ref()
            .and_then(|g| g.case)
            .as_deref()
            .copied()
    }

    /// Get parsed [TyKind] if any.
    pub fn ty(&self) -> Option<TyKind> {
        self.ty.map(|t| t.kind)
    }
}

impl ToTokens for KebabOutput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ty.to_tokens(tokens);
        self.combine_case_group.to_tokens(tokens);
    }
}

impl Parse for KebabOutput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(End) {
            Ok(Self::default())
        } else if let ty @ Some(_) = Ty::lookahead_parse(input, &lookahead)? {
            let lookahead = input.lookahead1();

            if lookahead.peek(End) {
                Ok(Self {
                    ty,
                    ..Default::default()
                })
            } else if MacroDelimiter::lookahead_peek(&lookahead) {
                Ok(Self {
                    ty,
                    combine_case_group: Some(input.parse()?),
                })
            } else {
                Err(lookahead.error())
            }
        } else if MacroDelimiter::lookahead_peek(&lookahead) {
            Ok(Self {
                combine_case_group: Some(input.parse()?),
                ..Default::default()
            })
        } else {
            Err(lookahead.error())
        }
    }
}

/// Combine and case spec of output.
#[derive(Debug)]
pub struct CombineCaseGroup {
    /// Bracket deliminating group.
    pub delim: MacroDelimiter,
    /// Case specified.
    pub case: Option<Case>,
    /// Combine specified.
    pub combine: Option<CombineKeyword>,
}

impl ToTokens for CombineCaseGroup {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.delim.surround(tokens, |tokens| {
            self.case.to_tokens(tokens);
            self.combine.to_tokens(tokens);
        });
    }
}

impl Parse for CombineCaseGroup {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let delim = macro_delimited!(content in input);
        let input = &content;

        let mut case = None;
        let mut combine = None;

        for val in do_n_times_then(
            2,
            || Ok(()),
            || Err(input.error("no further combine/case input expected")),
        ) {
            val?;

            let lookahead = input.lookahead1();

            if lookahead.peek(End) {
                break;
            }

            if combine.is_none() {
                if let value @ Some(_) = CombineKeyword::lookahead_parse(input, &lookahead)? {
                    combine = value;
                    continue;
                };
            }

            if case.is_none() {
                if let value @ Some(_) = Case::lookahead_parse(input, &lookahead)? {
                    case = value;
                    continue;
                }
            }

            return Err(lookahead.error());
        }

        Ok(Self {
            delim,
            case,
            combine,
        })
    }
}
