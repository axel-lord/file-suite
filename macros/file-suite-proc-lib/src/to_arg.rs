//! [ToArg] trait.

use ::std::rc::Rc;

use ::syn::punctuated::Punctuated;

use crate::punct_wrap::{Separated, Terminated};

/// Trait for parsed values which may be converted to arguments.
pub trait ToArg {
    /// Argument type self converts to.
    type Arg;

    /// Convert to argument.
    fn to_arg(&self) -> Self::Arg;
}

/// Used to implement [ToArg] for common collections of
/// arguments, such as [Punctuated], [Vec] and slices.
pub trait ToArgCollection
where
    Self: Sized + ToArg,
{
    /// Collection of arguments created by [ToArg] implementation.
    type Collection: FromIterator<Self::Arg>;
}

impl<T, P> ToArg for Punctuated<T, P>
where
    T: ToArgCollection,
{
    type Arg = T::Collection;

    fn to_arg(&self) -> Self::Arg {
        self.iter().map(ToArg::to_arg).collect()
    }
}

impl<T, P> ToArg for Terminated<T, P>
where
    T: ToArgCollection,
{
    type Arg = T::Collection;

    fn to_arg(&self) -> Self::Arg {
        self.0.iter().map(ToArg::to_arg).collect()
    }
}

impl<T, P> ToArg for Separated<T, P>
where
    T: ToArgCollection,
{
    type Arg = T::Collection;

    fn to_arg(&self) -> Self::Arg {
        self.0.iter().map(ToArg::to_arg).collect()
    }
}

impl<T> ToArg for Vec<T>
where
    T: ToArgCollection,
{
    type Arg = T::Collection;

    fn to_arg(&self) -> Self::Arg {
        self.iter().map(ToArg::to_arg).collect()
    }
}

impl<T> ToArg for Rc<[T]>
where
    T: ToArgCollection,
{
    type Arg = T::Collection;

    fn to_arg(&self) -> Self::Arg {
        self.iter().map(ToArg::to_arg).collect()
    }
}

impl<T> ToArg for [T]
where
    T: ToArgCollection,
{
    type Arg = T::Collection;

    fn to_arg(&self) -> Self::Arg {
        self.iter().map(ToArg::to_arg).collect()
    }
}

impl<const N: usize, T> ToArg for [T; N]
where
    T: ToArgCollection,
{
    type Arg = T::Collection;

    fn to_arg(&self) -> Self::Arg {
        self.iter().map(ToArg::to_arg).collect()
    }
}
