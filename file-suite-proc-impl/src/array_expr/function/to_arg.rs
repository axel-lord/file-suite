//! [ToArg] trait.

/// Trait for parsed values which may be converted to arguments.
pub trait ToArg {
    /// Argument type self converts to.
    type Arg;

    /// Convert to argument.
    fn to_arg(&self) -> Self::Arg;
}
