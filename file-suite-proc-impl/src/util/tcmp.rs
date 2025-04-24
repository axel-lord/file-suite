//! Compare TokenTrees to more easily created types.

use ::proc_macro2::TokenTree;

/// Trait to check equality against a [TokenTree].
pub trait TokenEq {
    /// Check equality against a token.
    fn token_cmp(&self, token: &TokenTree) -> bool;
}

/// Punctuation [TokenEq] proxy.
#[derive(Debug, Clone, Copy)]
pub struct P(pub char);

impl TokenEq for P {
    fn token_cmp(&self, token: &TokenTree) -> bool {
        if let TokenTree::Punct(pnct) = token {
            pnct.as_char() == self.0
        } else {
            false
        }
    }
}

/// Ident [TokenEq] proxy.
#[derive(Debug, Clone, Copy)]
pub struct I<'s>(pub &'s str);

impl TokenEq for I<'_> {
    fn token_cmp(&self, token: &TokenTree) -> bool {
        if let TokenTree::Ident(idnt) = token {
            idnt.to_string().as_str() == self.0
        } else {
            false
        }
    }
}

/// Create a Punctuation comparison sequence.
macro_rules! pseq {
    ($($p:literal),* $(,)?) => {
        [$(&$crate::util::tcmp::P($p)),*]
    };
}
pub(crate) use pseq;
