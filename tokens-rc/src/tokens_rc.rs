//! [TokensRc] impl.
use ::std::{fmt::Debug, ops::Deref, rc::Rc};

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::parse::Parse;

use crate::TokenRange;

/// Reference countet token array, providing cheap copying.
#[derive(Debug, Clone)]
pub struct TokensRc(Rc<[crate::TokenTree]>);

impl TokensRc {
    /// Get a subrange of the tokens as a [TokenStream].
    pub fn get_tokens<R>(&self, r: R) -> Option<TokenStream>
    where
        R: TokenRange,
    {
        let tokens = r.get(self)?;
        let mut token_stream = TokenStream::default();
        for token in tokens {
            token.to_tokens(&mut token_stream);
        }
        Some(token_stream)
    }
}

impl Deref for TokensRc {
    type Target = [crate::TokenTree];

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
