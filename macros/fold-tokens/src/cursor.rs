//! [Cursor] impl

use ::std::ops::Deref;

use ::tokens_rc::{TokenTree, TokensRc};

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

    /// Get a token relative to the current position.
    /// May be used to get prior tokens.
    pub fn get_relative(&self, by: isize) -> Option<&TokenTree> {
        let idx = if by < 0 {
            self.idx.checked_sub(by.unsigned_abs())?
        } else {
            self.idx.checked_add(by.unsigned_abs())?
        };

        self.tokens.get(idx)
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
