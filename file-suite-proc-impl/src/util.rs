//! Proc macro utilities.

pub mod group_help {
    //! Helpers for group parsing.

    pub use self::{delimited::Delimited, optional_delimited::OptionalDelimited};

    mod delimited;
    mod optional_delimited;
}
