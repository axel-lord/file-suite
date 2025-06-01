//! Utilities to fold tokens.

use ::std::{num::NonZero, ops::Deref};

use ::proc_macro2::{Literal, Punct, TokenStream};
use ::quote::{ToTokens, TokenStreamExt};
use ::syn::Ident;
use ::tokens_rc::{Group, TokenTree, TokensRc};

/// Cursor pointing to current token in a [TokensRc].
#[derive(Debug, Clone)]
pub struct Cursor {
    /// Current index.
    idx: usize,
    /// Wrapped tokens.
    tokens: TokensRc,
}

impl Deref for Cursor {
    type Target = [TokenTree];

    fn deref(&self) -> &Self::Target {
        self.tokens.get(self.idx..).unwrap_or_default()
    }
}

impl Cursor {
    /// Get a new token cursor.
    pub const fn new(tokens: TokensRc) -> Self {
        Self { idx: 0, tokens }
    }

    /// Move the cursor forwards
    pub const fn forward(&mut self, amount: usize) {
        self.idx = self.idx.saturating_add(amount);
    }

    /// Check if cursor matches a punctuation sequence.
    pub fn punct_match(&self, seq: &str) -> bool {
        let mut token_iter = self.iter();
        for chr in seq.chars() {
            let Some(TokenTree::Punct(punct)) = token_iter.next() else {
                return false;
            };
            if punct.as_char() != chr {
                return false;
            }
        }
        true
    }

    /// Check if cursor back matches a punctuation sequence.
    pub fn rpunct_match(&self, seq: &str) -> bool {
        let mut token_iter = self.iter().rev();
        for chr in seq.chars().rev() {
            let Some(TokenTree::Punct(punct)) = token_iter.next() else {
                return false;
            };
            if punct.as_char() != chr {
                return false;
            }
        }
        true
    }
}

/// Crate result type.
pub type Result<T = ()> = ::syn::Result<T>;

/// How to handle passed token.
#[derive(Debug, Clone, Copy)]
pub enum Response {
    /// Continue as normal, pushin the token to the output stream
    /// and further folding it if it is a group.
    Default,
    /// Skip the given amount of tokens.
    /// Should probably be used with a count of 1 if
    /// something alternative was pushed to the token stream.
    Skip(usize),
}

/// Trait for token folders to implement.
pub trait FoldTokens {
    /// Fold a punctuation token.
    ///
    /// # Errors
    /// The default implementation does not error
    /// however an implementor may whish to do
    /// so in some situations.
    fn fold_punct(
        &mut self,
        punct: &Punct,
        cursor: &Cursor,
        tokens: &mut TokenStream,
    ) -> Result<Response> {
        _ = (punct, cursor, tokens);
        Ok(Response::Default)
    }

    /// Fold a literal token.
    ///
    /// # Errors
    /// The default implementation does not error
    /// however an implementor may whish to do
    /// so in some situations.
    fn fold_literal(
        &mut self,
        literal: &Literal,
        cursor: &Cursor,
        tokens: &mut TokenStream,
    ) -> Result<Response> {
        _ = (literal, cursor, tokens);
        Ok(Response::Default)
    }

    /// Fold an ident token.
    ///
    /// # Errors
    /// The default implementation does not error
    /// however an implementor may whish to do
    /// so in some situations.
    fn fold_ident(
        &mut self,
        ident: &Ident,
        cursor: &Cursor,
        tokens: &mut TokenStream,
    ) -> Result<Response> {
        _ = (ident, cursor, tokens);
        Ok(Response::Default)
    }

    /// Fold an group token.
    ///
    /// # Errors
    /// The default implementation does not error
    /// however an implementor may whish to do
    /// so in some situations.
    fn fold_group(
        &mut self,
        group: &Group,
        cursor: &Cursor,
        tokens: &mut TokenStream,
    ) -> Result<Response> {
        _ = (group, cursor, tokens);
        Ok(Response::Default)
    }
}

/// Fold a [TokensRc] into a [TokenStream] using the given [FoldTokens] implementor.
///
/// # Errors
/// Should any fold function of `f` error, said error will be forwarded.
///
/// # Panics
/// If the [FoldTokens] implementor returns `Response::Skip(0)`.
pub fn fold_tokens(f: &mut dyn FoldTokens, tokens: TokensRc) -> Result<TokenStream> {
    let mut context = Vec::new();
    context.push((Cursor::new(tokens), TokenStream::default()));

    loop {
        let Some((cursor, tokens)) = context.last_mut() else {
            unreachable!()
        };

        match cursor.first() {
            Some(token) => {
                let response = match token {
                    TokenTree::Literal(literal) => f.fold_literal(literal, cursor, tokens),
                    TokenTree::Ident(ident) => f.fold_ident(ident, cursor, tokens),
                    TokenTree::Punct(punct) => f.fold_punct(punct, cursor, tokens),
                    TokenTree::Group(group) => f.fold_group(group, cursor, tokens),
                }?;

                match response {
                    Response::Default => match token {
                        TokenTree::Group(group) => {
                            let Group { stream, .. } = group.as_group();
                            let cursor = Cursor::new(stream.clone());
                            context.push((cursor, TokenStream::default()));
                        }
                        token => {
                            token.to_tokens(tokens);
                            cursor.forward(1)
                        }
                    },
                    Response::Skip(non_zero) => cursor.forward(
                        NonZero::new(non_zero)
                            .unwrap_or_else(|| panic!("skip amount should not be 0"))
                            .get(),
                    ),
                }
            }
            None => {
                let Some((_, tokens)) = context.pop() else {
                    unreachable!()
                };
                match context.last_mut() {
                    Some((cursor, tokens_lower)) => {
                        let Some(TokenTree::Group(group)) = cursor.first() else {
                            unreachable!()
                        };
                        let Group {
                            span, delimiter, ..
                        } = group.as_group();

                        let mut group = ::proc_macro2::Group::new(*delimiter, tokens);
                        group.set_span(*span);
                        tokens_lower.append(group);
                        cursor.forward(1);
                    }
                    None => break Ok(tokens),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use ::quote::{ToTokens, TokenStreamExt, quote};
    use ::syn::Ident;
    use ::tokens_rc::TokenTree;

    use crate::{FoldTokens, fold_tokens};

    #[derive(Debug, Clone, Copy)]
    pub struct F;

    impl FoldTokens for F {
        fn fold_ident(
            &mut self,
            ident: &syn::Ident,
            _cursor: &crate::Cursor,
            tokens: &mut proc_macro2::TokenStream,
        ) -> crate::Result<crate::Response> {
            if ident.to_string().as_str() == "Hello" {
                tokens.append(Ident::new("world", ident.span()));
                Ok(crate::Response::Skip(1))
            } else {
                Ok(crate::Response::Default)
            }
        }

        fn fold_punct(
            &mut self,
            punct: &proc_macro2::Punct,
            cursor: &crate::Cursor,
            tokens: &mut proc_macro2::TokenStream,
        ) -> crate::Result<crate::Response> {
            if punct.as_char() == '?' && cursor.punct_match("???") {
                if let Some(TokenTree::Group(g)) = cursor.get(3) {
                    g.stream.to_tokens(tokens);
                    Ok(crate::Response::Skip(4))
                } else {
                    Err(::syn::Error::new(
                        cursor[2].span(),
                        "expected delimited group following '???'",
                    ))
                }
            } else {
                Ok(crate::Response::Default)
            }
        }
    }

    #[test]
    fn fold() {
        let input = quote! {
            A B Hello C { 1 2 3 ( 4 5  ??? [ "msg" ] ) } ??? ( Unwrap )
        };
        let expected = quote! {
            A B world C { 1 2 3 ( 4 5 "msg" )} Unwrap
        };
        let result = fold_tokens(&mut F, input.into()).unwrap();

        assert_eq!(result.to_string(), expected.to_string());
    }
}
