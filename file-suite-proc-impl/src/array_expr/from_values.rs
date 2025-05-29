//! Trait and impl for converting value arrays to values.

use ::std::num::NonZero;

use crate::array_expr::{value::Value, value_array::ValueArray};

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

impl FromValues for Value {
    fn from_values(values: &[Value]) -> crate::Result<Self> {
        const EMPTY_MSG: &str = "cannot convert empty value array to a single value";
        const LARGE_MSG: &str =
            "cannot convert a value array of more than one value to a single value";
        match values {
            [value] => Ok(value.clone()),
            [] => Err(EMPTY_MSG.into()),
            [first, ..] => Err(first.span().map_or_else(
                || LARGE_MSG.into(),
                |span| ::syn::Error::new(span, LARGE_MSG).into(),
            )),
        }
    }
}

impl FromValues for ValueArray {
    fn from_values(values: &[Value]) -> crate::Result<Self> {
        Ok(values.iter().cloned().collect())
    }
}

impl FromValues for isize {
    fn from_values(values: &[Value]) -> crate::Result<Self> {
        Value::from_values(values)?
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
