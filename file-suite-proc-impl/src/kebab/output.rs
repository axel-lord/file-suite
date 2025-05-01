//! [Combine], [Case] and [Ty] impls.

use ::quote::ToTokens;
use ::syn::{
    bracketed,
    parse::{End, Parse},
    token::Bracket,
};

use crate::util::{do_n_times_then, kw_kind};

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
    pub fn combine(&self) -> Option<CombineKind> {
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
            } else if lookahead.peek(Bracket) {
                Ok(Self {
                    ty,
                    combine_case_group: Some(input.parse()?),
                })
            } else {
                Err(lookahead.error())
            }
        } else if lookahead.peek(Bracket) {
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
    pub bracket: Bracket,
    /// Case specified.
    pub case: Option<Case>,
    /// Combine specified.
    pub combine: Option<Combine>,
}

impl ToTokens for CombineCaseGroup {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.bracket.surround(tokens, |tokens| {
            self.case.to_tokens(tokens);
            self.combine.to_tokens(tokens);
        });
    }
}

impl Parse for CombineCaseGroup {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let bracket = bracketed!(content in input);
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
                if let value @ Some(_) = Combine::lookahead_parse(input, &lookahead)? {
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
            bracket,
            case,
            combine,
        })
    }
}

kw_kind!(
    Combine
    /// How output should be combined.
    CombineKind (Default) {
        /// Values should be concatenated without any separator.
        [default]
        Concat concat,
        /// Values should be joined by a dash,
        Kebab kebab,
        /// Values should be joined by an underscore.
        Snake snake,
        /// Values should be joined by a space.
        Space space,
    }
);

impl CombineKind {
    /// Join input arguments.
    pub fn join(self, values: Vec<String>) -> String {
        values.join(match self {
            CombineKind::Concat => "",
            CombineKind::Kebab => "-",
            CombineKind::Snake => "_",
            CombineKind::Space => " ",
        })
    }

    /// Preferred [TyKind] of variant.
    pub const fn default_ty(self) -> Option<TyKind> {
        Some(match self {
            Self::Space | Self::Kebab => TyKind::LitStr,
            _ => return None,
        })
    }
}

kw_kind!(
    /// A parsed output case (has span).
    Case
    /// How output case should be modified.
    CaseKind (Default) {
        /// Keep case as is.
        [default]
        Keep keep,
        /// Use camelCase.
        Camel camel,
        /// Use PascalCase.
        Pascal pascal,
        /// Use UPPERCASE.
        Upper upper,
        /// Use LOWERCASE.
        Lower lower,
    }
);

impl CaseKind {
    /// Apply casing to a string.
    pub fn apply(self, mut values: Vec<String>) -> Vec<String> {
        fn titlecase(value: &str) -> String {
            let mut chars = value.chars();
            chars
                .next()
                .map(|first| first.to_uppercase())
                .into_iter()
                .flatten()
                .chain(chars.flat_map(char::to_lowercase))
                .collect()
        }
        match self {
            CaseKind::Keep => (),
            CaseKind::Camel => {
                let mut values = values.iter_mut();
                if let Some(first) = values.next() {
                    *first = first.to_lowercase();
                }
                for value in values {
                    *value = titlecase(value);
                }
            }
            CaseKind::Pascal => {
                for value in values.iter_mut() {
                    *value = titlecase(value);
                }
            }
            CaseKind::Upper => {
                for value in values.iter_mut() {
                    *value = value.to_uppercase();
                }
            }
            CaseKind::Lower => {
                for value in values.iter_mut() {
                    *value = value.to_lowercase();
                }
            }
        };
        values
    }
}

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
