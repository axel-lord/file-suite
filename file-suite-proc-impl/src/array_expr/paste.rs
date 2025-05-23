//! Find array expressions in any token input.

use ::std::cell::Cell;

use ::proc_macro2::TokenTree;
use ::quote::ToTokens;
use ::syn::parse::Parser;

use crate::{
    array_expr::{Node, storage::Storage},
    util::{
        fold_tokens::{FoldTokens, fold_punct},
        tcmp::pseq,
    },
};

/// [FoldTokens] for finding array expressions.
#[derive(Debug, Default)]
pub(crate) struct ArrayExprPaste {
    /// Variable storage for paste expression.
    storage: Storage,
}

impl FoldTokens<2> for ArrayExprPaste {
    fn fold_punct(
        &mut self,
        punct: proc_macro2::Punct,
        lookahead: &mut crate::util::TokenLookahead<proc_macro2::TokenStream, 2>,
        tokens: &mut proc_macro2::TokenStream,
    ) -> syn::Result<()> {
        if !lookahead.matches_after(punct.clone(), pseq!('+', '+', '!')) {
            fold_punct(punct, tokens);
            return Ok(());
        }
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
                ::syn::Error::new(span.get(), "expected delimited group following '++!'")
            })?;

        self.storage.with_local_layer(|storage| {
            for node in Node::parse_multiple.parse2(group.stream())? {
                for value in node.to_array_expr().compute_with_storage(storage)? {
                    value.try_to_typed()?.to_tokens(tokens);
                }
            }

            Ok(())
        })
    }
}
