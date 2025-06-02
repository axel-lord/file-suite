//! Find array expressions in any token input.

use ::fold_tokens::{Cursor, Response};
use ::proc_macro2::{Punct, TokenStream};
use ::quote::ToTokens;
use ::syn::parse::Parser;
use ::tokens_rc::TokenTree;

use crate::{ParsedArrayExpr, storage::Storage};

/// [FoldTokens] for finding array expressions.
#[derive(Debug)]
pub(crate) struct ArrayExprPaste<'s> {
    /// Variable storage for paste expression.
    pub storage: &'s mut Storage,
}

impl ::fold_tokens::FoldTokens for ArrayExprPaste<'_> {
    fn fold_punct(
        &mut self,
        punct: &Punct,
        cursor: &Cursor,
        tokens: &mut TokenStream,
    ) -> fold_tokens::Result<Response> {
        if punct.as_char() == '#' {
            let Some(tree) = cursor.get(1) else {
                return Ok(Response::default());
            };

            let mut skip = 2;
            let key = match tree {
                TokenTree::Literal(literal) => match ::syn::Lit::new(literal.clone()) {
                    ::syn::Lit::Str(lit_str) => lit_str.value(),
                    ::syn::Lit::Int(lit_int) => lit_int.base10_parse::<isize>()?.to_string(),
                    _ => return Ok(Response::Default),
                },
                TokenTree::Ident(ident) => ident.to_string(),
                TokenTree::Punct(punct) => match cursor.get(2) {
                    Some(TokenTree::Literal(lit)) if punct.as_char() == '-' => {
                        let ::syn::Lit::Int(lit_int) = ::syn::Lit::new(lit.clone()) else {
                            return Ok(Response::Default);
                        };
                        skip = 3;
                        lit_int.base10_parse::<isize>()?.to_string()
                    }
                    _ => return Ok(Response::Default),
                },
                TokenTree::Group(..) => {
                    return Ok(Response::Default);
                }
            };
            let skip = skip;

            for value in self.storage.try_get(&key)? {
                value.try_to_typed()?.to_tokens(tokens);
            }

            Ok(Response::Skip(skip))
        } else if cursor.punct_match("++!") {
            let Some(::tokens_rc::TokenTree::Group(group)) = cursor.get(3) else {
                return Err(::syn::Error::new(
                    cursor[2].span(),
                    "expected delimited group following '++!'",
                ));
            };

            for node in ParsedArrayExpr::parse_multiple.parse2(group.stream())? {
                for value in self.storage.with_local_layer(|storage| {
                    node.to_array_expr().compute_with_storage(storage)
                })? {
                    value.try_to_typed()?.to_tokens(tokens);
                }
            }

            Ok(Response::Skip(4))
        } else {
            Ok(Response::Default)
        }
    }
}
