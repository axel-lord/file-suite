//! [KebabPaste] impl.

use ::std::cell::Cell;

use ::proc_macro2::TokenTree;
use ::quote::ToTokens;
use ::syn::parse::Parser;

use crate::{
    kebab::kebab_inner,
    util::{
        fold_tokens::{FoldTokens, fold_punct},
        tcmp::pseq,
    },
};

/// [FoldTokens] for finding and substituting kebab expressions.
#[derive(Debug)]
pub(super) struct KebabPaste;

impl FoldTokens<2> for KebabPaste {
    fn fold_punct(
        &mut self,
        punct: proc_macro2::Punct,
        lookahead: &mut crate::util::TokenLookahead<proc_macro2::TokenStream, 2>,
        tokens: &mut proc_macro2::TokenStream,
    ) -> syn::Result<()> {
        if !(punct.as_char() == '-' && lookahead.matches(pseq!('-', '!'))) {
            fold_punct(punct, tokens);
            return Ok(());
        };

        let span = Cell::new(
            lookahead
                .discard()
                .next_back()
                .unwrap_or_else(|| unreachable!())
                .span(),
        );

        let group = lookahead
            .next()
            .and_then(|tree| {
                span.set(tree.span());
                if let TokenTree::Group(group) = tree {
                    Some(group)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                ::syn::Error::new(span.get(), "expected delimited group following '--!'")
            })?;

        for value in kebab_inner.parse2(group.stream())? {
            value.try_to_typed()?.to_tokens(tokens);
        }

        Ok(())
    }
}
