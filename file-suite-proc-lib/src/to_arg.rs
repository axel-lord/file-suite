//! [ToArg] trait.

use ::std::rc::Rc;

use ::syn::punctuated::Punctuated;

/// Trait for parsed values which may be converted to arguments.
pub trait ToArg {
    /// Argument type self converts to.
    type Arg;

    /// Convert to argument.
    fn to_arg(&self) -> Self::Arg;
}

/// Trait for parsed values which when puncutated may be converted to arguments.
pub trait PunctuatedToArg
where
    Self: Sized,
{
    /// Argument type puctuated list of self converts to.
    type Arg;

    /// Convert to argument.
    fn punctuated_to_arg<P>(punctuated: &Punctuated<Self, P>) -> Self::Arg;
}

/// Trait for parsed values which when in a slice may be converted to arguments.
pub trait SliceToArg
where
    Self: Sized,
{
    /// Argument type puctuated list of self converts to.
    type Arg;

    /// Convert to argument.
    fn slice_to_arg(slice: &[Self]) -> Self::Arg;
}

impl<T, P> ToArg for Punctuated<T, P>
where
    T: PunctuatedToArg,
{
    type Arg = <T as PunctuatedToArg>::Arg;

    fn to_arg(&self) -> Self::Arg {
        T::punctuated_to_arg(self)
    }
}

impl<T> ToArg for Vec<T>
where
    T: SliceToArg,
{
    type Arg = <T as SliceToArg>::Arg;

    fn to_arg(&self) -> Self::Arg {
        T::slice_to_arg(self)
    }
}

impl<T> ToArg for Rc<[T]>
where
    T: SliceToArg,
{
    type Arg = <T as SliceToArg>::Arg;

    fn to_arg(&self) -> Self::Arg {
        T::slice_to_arg(self)
    }
}

impl<T> ToArg for [T]
where
    T: SliceToArg,
{
    type Arg = <T as SliceToArg>::Arg;

    fn to_arg(&self) -> Self::Arg {
        T::slice_to_arg(self)
    }
}

impl<const N: usize, T> ToArg for [T; N]
where
    T: SliceToArg,
{
    type Arg = <T as SliceToArg>::Arg;

    fn to_arg(&self) -> Self::Arg {
        T::slice_to_arg(self)
    }
}
