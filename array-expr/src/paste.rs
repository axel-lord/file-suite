//! Find array expressions in any token input.

use ::fold_tokens::{Cursor, Response};
use ::proc_macro2::{Punct, TokenStream};
use ::quote::ToTokens;
use ::syn::parse::Parser;

use crate::{Node, storage::Storage};

/// [FoldTokens] for finding array expressions.
#[derive(Debug)]
pub(crate) struct ArrayExprPaste<'s> {
    /// Variable storage for paste expression.
    pub storage: &'s mut Storage,
}

impl ::fold_tokens::FoldTokens for ArrayExprPaste<'_> {
    fn fold_punct(
        &mut self,
        _punct: &Punct,
        cursor: &Cursor,
        tokens: &mut TokenStream,
    ) -> fold_tokens::Result<Response> {
        if !cursor.punct_match("++!") {
            return Ok(Response::Default);
        }

        let Some(::tokens_rc::TokenTree::Group(group)) = cursor.get(3) else {
            return Err(::syn::Error::new(
                cursor[2].span(),
                "expected delimited group following '++!'",
            ));
        };

        for node in Node::parse_multiple.parse2(group.stream())? {
            for value in self
                .storage
                .with_local_layer(|storage| node.to_array_expr().compute_with_storage(storage))?
            {
                value.try_to_typed()?.to_tokens(tokens);
            }
        }

        Ok(Response::Skip(4))
    }
}
