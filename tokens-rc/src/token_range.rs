//! [TokenRange] trait impl.

use ::std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use crate::TokenTree;

/// Get a range of a token rc.
pub trait TokenRange {
    /// Get a slice using specified range.
    fn get(self, tokens: &[TokenTree]) -> Option<&[TokenTree]>;
}

/// Implement for ranges.
macro_rules! impl_token_range {
    ($($t:ty),*) => {$(
        impl TokenRange for $t {
            fn get(self, tokens: &[TokenTree]) -> Option<&[TokenTree]> {
                tokens.get(self)
            }
        }
    )*};
}
impl_token_range!(
    Range<usize>,
    RangeFull,
    RangeTo<usize>,
    RangeFrom<usize>,
    RangeInclusive<usize>,
    RangeToInclusive<usize>
);
