//! Proc macro utilities.

pub use self::lookahead::TokenLookahead;

mod lookahead;

pub mod delimited;
pub mod fold_tokens;
pub mod tcmp;

pub mod group_help {
    //! Helpers for group parsing.

    pub use self::{
        delimited::Delimited, empty_delimited::EmptyDelimited,
        optional_delimited::OptionalDelimited,
    };

    mod delimited;
    mod empty_delimited;
    mod optional_delimited;
}
