//! Proc macro utilities.

use ::std::iter;

pub(crate) use self::{
    delimited::{MacroDelimExt, macro_delimited},
    kw_kind::kw_kind,
};

pub use self::lookahead::TokenLookahead;

mod lookahead;

mod kw_kind;

mod delimited;

pub mod fold_tokens;

pub mod tcmp;

/// Create an iterator that repeats a function result n times then yields the result of the
/// termination function.
pub(crate) fn do_n_times_then<T>(
    n: usize,
    repeater: impl FnMut() -> T,
    term: impl FnOnce() -> T,
) -> impl Iterator<Item = T> {
    iter::repeat_with(repeater)
        .take(n)
        .chain(iter::once_with(term))
}
