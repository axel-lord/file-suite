//! [Function] impl.

use ::std::fmt::Debug;

pub(crate) use macros::{function_enum, function_struct, spec_impl};

pub mod builtin {
    //! Builtin funtions.

    pub mod case;
    pub mod count;
    pub mod enumerate;
    pub mod join;
    pub mod rev;
    pub mod set;
    pub mod split;
    pub mod ty;
}

use crate::array_expr::function::builtin::set::{global, local};

pub use self::call::{Call, ToCallable};

use self::builtin::{
    case::case, count::count, enumerate::enumerate, join::join, rev::rev, split::split, ty::ty,
};

mod macros;

mod call;

function_enum!(
    /// Enum collecting [Call] implementors.
    #[derive(Debug, Clone)]
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
        /// Set a global variable.
        Global(global),
        /// Set a local variable.
        Local(local),
    }
);
