//! Conversion utilities using [Any]

use ::std::any::Any;

/// Trait for types which may be converted to from a boxed [Any] value.
pub trait FromAny: Sized {
    /// Try to convert a boxed [Any] to self.
    ///
    /// # Errors
    /// If the boxed [Any] value does not meet the expectations of Self, the boxed [Any] value
    /// is returned.
    fn try_from_any(value: Box<dyn Any>) -> Result<Self, Box<dyn Any>>;

    /// Convert a boxed [Any] to self.
    ///
    /// # Panics
    /// If the boxed any cannot be converted to Self.
    fn from_any(value: Box<dyn Any>) -> Self {
        Self::try_from_any(value)
            .unwrap_or_else(|_| panic!("could not convert boxed Any value to instance"))
    }

    /// Try to convert a value of a type implementing [Any] to Self.
    ///
    /// # Errors
    /// If the [Any] value does not meet the expectations of Self, it is returned.
    ///
    /// # Panics
    /// If the [FromAny::try_from_any] implementation errors and does not return a boxed [Any]
    /// value with the same type as passed to this function.
    fn try_from_any_value<A>(value: A) -> Result<Self, A>
    where
        A: Any,
    {
        match Self::try_from_any(Box::new(value)) {
            Ok(self_value) => Ok(self_value),
            Err(value) => {
                if let Ok(value) = value.downcast::<A>() {
                    Err(*value)
                } else {
                    panic!(
                        "type of try_from_any returned boxed Any value does not match type of input Any value"
                    )
                }
            }
        }
    }

    /// Convert afrom a value of a type implementing [Any] to Self.
    ///
    /// # Panics
    /// If the value cannot be converted to Self while boxed.
    fn from_any_value<A>(value: A) -> Self
    where
        A: Any,
    {
        Self::from_any(Box::new(value))
    }
}

/// Trait fro types which may be converted into a boxed [Any] value.
pub trait IntoAny: Sized {
    /// Convert a value to a boxed [Any] instance.
    fn into_any(self) -> Box<dyn Any>;

    /// Try to convert Self to another value implementing [FromAny].
    ///
    /// # Errors
    /// If [FromAny::try_from_any] fails the intermediate boxed [Any] is returned.
    fn try_any_conv<T>(self) -> Result<T, Box<dyn Any>>
    where
        T: FromAny,
    {
        T::try_from_any(self.into_any())
    }

    /// Convert Self to another value implementing [FromAny].
    ///
    /// # Panics
    /// If [FromAny::from_any] panics due to incompatabilities.
    fn any_conv<T>(self) -> T
    where
        T: FromAny,
    {
        T::from_any(self.into_any())
    }
}
