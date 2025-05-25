//! [Function] impl.

use ::std::fmt::Debug;

pub(crate) use macros::{function_enum, function_struct, spec_impl};

pub mod builtin {
    //! Builtin funtions.

    pub mod alias;
    pub mod case;
    pub mod chunk;
    pub mod clear;
    pub mod count;
    pub mod enumerate;
    pub mod join;
    pub mod rev;
    pub mod set;
    pub mod shift;
    pub mod split;
    pub mod trim;
    pub mod ty;
    pub mod use_alias;
}

use crate::array_expr::function::builtin::{
    alias::alias,
    case::case,
    chunk::chunk,
    clear::clear,
    count::count,
    enumerate::enumerate,
    join::join,
    rev::rev,
    set::{global, local},
    shift::shift,
    split::split,
    trim::trim,
    ty::ty,
    use_alias::UseAlias,
};

pub use self::{
    call::{Call, ToCallable},
    chain::FunctionChain,
};

mod macros;

mod call;

mod chain;

/// Type used in call chains, result of [ToCallable] on [Function].
pub type FunctionCallable = <Function as ToCallable>::Call;

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
        /// Trim array elements.
        Trim(trim),
        /// Shift/Rotate elements.
        Shift(shift),
        /// Count array elements.
        Count(count),
        /// Split array into chunks.
        Chunks(chunk),
        /// Clear array.
        Clear(clear),
        /// Set a global variable.
        Global(global),
        /// Set a local variable.
        Local(local),
        /// Set an alias.
        Alias(alias),
        /// Use an alias.
        UseAlias(UseAlias),
    }
);
