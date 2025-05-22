//! [Function] impl.

use ::std::fmt::Debug;

pub(crate) use macros::{function_enum, function_struct, spec_impl};

mod builtin {
    //! Builtin funtions.

    pub mod case;
    pub mod count;
    pub mod enumerate;
    pub mod join;
    pub mod rev;
    pub mod split;
    pub mod ty;
}

pub use self::{
    builtin::{
        case::case, count::count, enumerate::enumerate, join::join, rev::rev, split::split, ty::ty,
    },
    call::{Call, ToCallable},
};

mod macros;

mod call;

function_enum!(
    /// Enum collecting [Call] implementors.
    #[derive(Debug)]
    Function {
        /// Split array according to specification
        Split(split),
        /// Join array according to specification.
        Join(join),
        /// Case array according to specification.
        Case(case),
        /// Convert type of array.
        Type(ty),
        /// Enumerate array.
        Enumerate(enumerate),
        /// Reverse array.
        Rev(rev),
        /// Count array elements.
        Count(count),
    }
);
