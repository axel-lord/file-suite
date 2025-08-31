//! [FromArg] trait.

use crate::ToArg;

/// Trait for callables which may be created from a single argument.
pub trait FromArg {
    /// Parsed argument type.
    type Factory: ToArg;

    /// Create the value from an argument.
    fn from_arg(arg: ArgTy<Self>) -> Self;
}

/// Arument type of [FromArg] implementor.
pub type ArgTy<T> = <<T as FromArg>::Factory as ToArg>::Arg;
