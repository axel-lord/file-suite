//! Traits for functions.

use ::std::borrow::Cow;

use crate::array_expr::value_array::ValueArray;

/// Trait for items which may be converted to a [Call] implementor.
pub trait ToCallable {
    /// Callable to convert into.
    type Call: Call;

    /// Convert to a callable.
    fn to_callable(&self) -> Self::Call;
}

/// Trait for functions which may transform a vec of values.
pub trait Call {
    /// Transform the passed input.
    ///
    /// # Errors
    /// If input may not be transformed according to specification.
    fn call(&self, input: ValueArray) -> Result<ValueArray, Cow<'static, str>>;
}
