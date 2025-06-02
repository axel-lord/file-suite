//! Find array expressions in any token input.

use ::std::collections::HashSet;

use ::fold_tokens::{Cursor, FoldTokens, Response};
use ::proc_macro2::{Punct, TokenStream};
use ::quote::ToTokens;
use ::syn::parse::Parser;
use ::tokens_rc::TokenTree;

use crate::{ParsedArrayExpr, storage::Storage};

/// [VisitTokens] to find variables that are interpolated.
#[derive(Debug)]
struct FindVars {
    /// Found variables.
    vars: HashSet<String>,
}

/// [FoldTokens] for finding array expressions.
#[derive(Debug)]
pub(crate) struct ArrayExprPaste<'s> {
    /// Variable storage for paste expression.
    pub storage: &'s mut Storage,
}

impl FoldTokens for ArrayExprPaste<'_> {
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
                TokenTree::Group(group) => {
                    match group.delimiter() {
                        ::proc_macro2::Delimiter::Parenthesis => (),
                        _ => return Ok(Response::Default),
                    }

                    let sep = match cursor.get(2) {
                        Some(TokenTree::Punct(p)) if p.as_char() == '*' => None,
                        Some(TokenTree::Punct(p)) => match cursor.get(3) {
                            Some(TokenTree::Punct(p2)) if p2.as_char() == '*' => Some(p),
                            _ => return Ok(Response::Default),
                        },
                        _ => return Ok(Response::Default),
                    };

                    return Ok(Response::Skip(sep.map_or_else(|| 3, |_| 4)));
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
