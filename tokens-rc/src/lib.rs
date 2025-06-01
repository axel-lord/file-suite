//! Reference countet array of tokens.

use ::std::{
    cell::{Cell, OnceCell},
    fmt::Debug,
    ops::Deref,
    rc::Rc,
};

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
    Group(OpaqueGroup),
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

/// Wrapper for [Group] with lazy conversion from [::proc_macro2::Group].
pub struct OpaqueGroup {
    /// Wrapped group.
    group: OnceCell<Group>,
    /// Group to lazily convert from.
    backing: Cell<Option<::proc_macro2::Group>>,
}

impl OpaqueGroup {
    /// Get a reference to wrapped group.
    pub fn as_group(&self) -> &Group {
        let Self { group, backing } = self;

        group.get_or_init(|| Group::from(backing.take().unwrap_or_else(|| unreachable!())))
    }

    /// Get span of opaque group.
    pub fn span(&self) -> Span {
        if let Some(group) = self.backing.take() {
            let span = group.span();
            self.backing.set(Some(group));
            span
        } else {
            self.as_group().span
        }
    }

    /// Get tokens contained by group as a [TokenStream].
    pub fn stream(&self) -> TokenStream {
        if let Some(group) = self.backing.take() {
            let stream = group.stream();
            self.backing.set(Some(group));
            stream
        } else {
            self.stream.to_token_stream()
        }
    }
}

impl Debug for OpaqueGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let backing = self.backing.take();
        let res = f
            .debug_struct("OpaqueGroup")
            .field("group", &self.group)
            .field("backing", &backing)
            .finish();
        self.backing.set(backing);
        res
    }
}

impl Clone for OpaqueGroup {
    fn clone(&self) -> Self {
        Self {
            group: OnceCell::from(self.as_group().clone()),
            backing: Cell::new(None),
        }
    }
}

impl Deref for OpaqueGroup {
    type Target = Group;

    fn deref(&self) -> &Self::Target {
        self.as_group()
    }
}

impl From<::proc_macro2::Group> for OpaqueGroup {
    fn from(value: ::proc_macro2::Group) -> Self {
        Self {
            group: OnceCell::new(),
            backing: Cell::new(Some(value)),
        }
    }
}

impl From<Group> for OpaqueGroup {
    fn from(value: Group) -> Self {
        Self {
            group: OnceCell::from(value),
            backing: Cell::new(None),
        }
    }
}

impl From<OpaqueGroup> for Group {
    fn from(value: OpaqueGroup) -> Self {
        let OpaqueGroup { group, backing } = value;

        if let Some(group) = backing.into_inner() {
            Group::from(group)
        } else {
            group.into_inner().unwrap_or_else(|| unreachable!())
        }
    }
}

impl ToTokens for OpaqueGroup {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(group) = self.backing.take() {
            group.to_tokens(tokens);
            self.backing.set(Some(group));
        } else {
            self.as_group().to_tokens(tokens);
        }
    }
}
