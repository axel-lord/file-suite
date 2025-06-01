//! Reference countet array of tokens.

use ::std::{ops::Deref, rc::Rc};

use ::proc_macro2::{Delimiter, Literal, Punct, Span, TokenStream, extra::DelimSpan};
use ::quote::{ToTokens, TokenStreamExt};
use ::syn::{Ident, parse::Parse};

/// Reference countet token array, providing cheap copying.
#[derive(Debug, Clone)]
pub struct TokensRc(Rc<[TokenTree]>);

impl Deref for TokensRc {
    type Target = [TokenTree];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToTokens for TokensRc {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for tree in self.0.iter() {
            tree.to_tokens(tokens);
        }
    }
}

impl From<TokenStream> for TokensRc {
    fn from(value: TokenStream) -> Self {
        let mut tokens = Vec::new();
        for token in value {
            tokens.push(token.into());
        }
        Self(tokens.into())
    }
}

impl Parse for TokensRc {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<TokenStream>().map(Self::from)
    }
}

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
    Group(Group),
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

/// [Group][::proc_macro2::Group] replacement for [TokensRc].
#[derive(Debug, Clone)]
pub struct Group {
    /// Span of group.
    pub span: Span,
    /// Span of group delims.
    pub delim_span: DelimSpan,
    /// Group delimiter.
    pub delimiter: Delimiter,
    /// Tokens of group.
    pub stream: TokensRc,
}

impl ToTokens for Group {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let stream = self.stream.to_token_stream();
        let mut group = ::proc_macro2::Group::new(self.delimiter, stream);
        group.set_span(self.span);
        tokens.append(group);
    }
}
impl From<::proc_macro2::Group> for Group {
    fn from(value: ::proc_macro2::Group) -> Self {
        Self {
            span: value.span(),
            delim_span: value.delim_span(),
            delimiter: value.delimiter(),
            stream: value.stream().into(),
        }
    }
}
