//! [FromArg] trait.

use crate::array_expr::function::ToArg;

/// Trait for callables which may be created from a single argument.
pub trait FromArg {
    /// Parsed argument type.
    type ArgFactory: ToArg;

    /// Create the value from an argument.
    fn from_arg(arg: ArgTy<Self>) -> Self;
}

/// Arument type of [FromtArg] implementor.
pub type ArgTy<T> = <<T as FromArg>::ArgFactory as ToArg>::Arg;
