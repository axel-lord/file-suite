//! Trait and impl for converting value arrays to values.

use ::std::num::NonZero;

use crate::{value::Value, value_array::ValueArray};

/// Trait for types which may be created from a slice of [Value].
pub trait FromValues
where
    Self: Sized,
{
    /// Convert a slice of [Value] to self.
    ///
    /// # Errors
    /// If self cannot be created from the given values.
    fn from_values(values: &[Value]) -> crate::Result<Self>;
}

/// Message given when trying to convert from a value array where there
/// should be at least one value.
const EMPTY_MSG: &str = "value array should not be empty";
/// Message given when trying to convert from a value array where there shoulf be only one value
const LARGE_MSG: &str = "value array should only contain one value";

/// Get a single value from a slice of values.
///
/// # Errors
/// If the value slice does not contain one, and only one, value.
pub fn ensure_single(values: &[Value]) -> crate::Result<&Value> {
    match values {
        [value] => Ok(value),
        [] => Err(EMPTY_MSG.into()),
        [first, ..] => Err(first.span().map_or_else(
            || LARGE_MSG.into(),
            |span| ::syn::Error::new(span, LARGE_MSG).into(),
        )),
    }
}

/// Get a string slice from a slice of values.
///
/// # Errors
/// If the value slice does not contain one, and only one, value.
pub fn str_from_values(values: &[Value]) -> crate::Result<&str> {
    ensure_single(values).map(|value| value.as_str())
}

impl FromValues for Value {
    fn from_values(values: &[Value]) -> crate::Result<Self> {
        ensure_single(values).cloned()
    }
}

impl FromValues for ValueArray {
    fn from_values(values: &[Value]) -> crate::Result<Self> {
        Ok(values.iter().cloned().collect())
    }
}

impl FromValues for isize {
    fn from_values(values: &[Value]) -> crate::Result<Self> {
        ensure_single(values)?
            .get_int()
            .map_err(crate::Error::from_display)
    }
}

impl FromValues for usize {
    fn from_values(values: &[Value]) -> crate::Result<Self> {
        isize::from_values(values)?
            .try_into()
            .map_err(crate::Error::from_display)
    }
}

impl FromValues for NonZero<isize> {
    fn from_values(values: &[Value]) -> crate::Result<Self> {
        isize::from_values(values)
            .map(NonZero::new)
            .transpose()
            .ok_or("value is not non-zero")?
    }
}

impl FromValues for NonZero<usize> {
    fn from_values(values: &[Value]) -> crate::Result<Self> {
        usize::from_values(values)
            .map(NonZero::new)
            .transpose()
            .ok_or("value is not non-zero")?
    }
}

impl FromValues for String {
    fn from_values(values: &[Value]) -> crate::Result<Self> {
        str_from_values(values).map(String::from)
    }
}
