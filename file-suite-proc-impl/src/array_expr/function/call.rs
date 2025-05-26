//! Traits for functions.

use crate::array_expr::{storage::Storage, value_array::ValueArray};

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
    fn call(&self, array: ValueArray, storage: &mut Storage) -> crate::Result<ValueArray>;
}

impl<T> ToCallable for Option<T>
where
    T: ToCallable,
    T::Call: Default,
{
    type Call = T::Call;

    fn to_callable(&self) -> Self::Call {
        self.as_ref().map(|t| t.to_callable()).unwrap_or_default()
    }
}
