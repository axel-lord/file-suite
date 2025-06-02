//! [TokenTree] impl.

use ::proc_macro2::{Literal, Punct, Span};
use ::quote::ToTokens;
use ::syn::Ident;

/// [TokenTree][::proc_macro2::TokenTree] replacement for [TokensRc]
#[derive(Debug, Clone)]
pub enum TokenTree {
    /// Literal token.
    Literal(Literal),
    /// Identifier token.
    Ident(Ident),
    /// Punctuation token.
    Punct(Punct),
    /// Group tokens.
    Group(crate::OpaqueGroup),
}

impl TokenTree {
    /// Get span of token tree.
    pub fn span(&self) -> Span {
        match self {
            TokenTree::Literal(literal) => literal.span(),
            TokenTree::Ident(ident) => ident.span(),
            TokenTree::Punct(punct) => punct.span(),
            TokenTree::Group(opaque_group) => opaque_group.span(),
        }
    }
}

impl From<::proc_macro2::TokenTree> for TokenTree {
    fn from(value: ::proc_macro2::TokenTree) -> Self {
        match value {
            ::proc_macro2::TokenTree::Group(group) => Self::Group(group.into()),
            ::proc_macro2::TokenTree::Ident(ident) => Self::Ident(ident),
            ::proc_macro2::TokenTree::Punct(punct) => Self::Punct(punct),
            ::proc_macro2::TokenTree::Literal(literal) => Self::Literal(literal),
        }
    }
}

impl ToTokens for TokenTree {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            TokenTree::Literal(literal) => literal.to_tokens(tokens),
            TokenTree::Ident(ident) => ident.to_tokens(tokens),
            TokenTree::Punct(punct) => punct.to_tokens(tokens),
            TokenTree::Group(group) => group.to_tokens(tokens),
        }
    }
}

