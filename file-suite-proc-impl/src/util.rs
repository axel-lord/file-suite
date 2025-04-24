//! Proc macro utilities.

pub use self::{any_of::Either, lookahead::token_lookahead};

mod any_of;

mod lookahead;

pub mod tcmp;
