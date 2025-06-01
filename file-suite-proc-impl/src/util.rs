//! Proc macro utilities.

pub use self::lookahead::TokenLookahead;

mod lookahead;

pub mod fold_tokens;
pub mod tcmp;

pub mod group_help {
    //! Helpers for group parsing.

    pub use self::{delimited::Delimited, optional_delimited::OptionalDelimited};

    mod delimited;
    mod optional_delimited;
}
