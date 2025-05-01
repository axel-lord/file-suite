//! Proc macro utilities.

pub(crate) use self::{
    any_of::{AnyOf3, Either},
    kw_kind::kw_kind,
    lookahead::token_lookahead,
};

mod any_of;

mod lookahead;

mod kw_kind;

pub mod tcmp;
