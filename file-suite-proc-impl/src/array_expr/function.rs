//! [Function] impl.

use ::std::fmt::Debug;

use crate::array_expr::{function::count::Count, value_array::ValueArray};

pub use crate::array_expr::function::{
    case::Case, enumerate::Enumerate, join::Join, rev::Rev, split::Split, ty::Type,
};

mod split;

mod join;

mod case;

mod ty;

mod rev;

mod enumerate;

mod count;

/// Trait for items which may be converted to a [Call] implementor.
pub trait ToCallable {
    /// Callable to convert into.
    type Call: Call;

    /// Convert to a callable.
    fn to_callable(&self) -> Self::Call;
}

impl<C> ToCallable for C
where
    C: Call + Clone,
{
    type Call = Self;

    #[inline]
    fn to_callable(&self) -> Self::Call {
        self.clone()
    }
}

/// Trait for functions which may transform a vec of values.
pub trait Call {
    /// Transform the passed input.
    ///
    /// # Errors
    /// If input may not be transformed according to specification.
    fn call(&self, input: ValueArray) -> ::syn::Result<ValueArray>;
}

function_enum!(
    /// Enum collecting [Call] implementors.
    #[derive(Debug)]
    Function {
        /// Split array according to specification
        Split(Split),
        /// Join array according to specification.
        Join(Join),
        /// Case array according to specification.
        Case(Case),
        /// Convert type of array.
        Type(Type),
        /// Enumerate array.
        Enumerate(Enumerate),
        /// Reverse array.
        Rev(Rev),
        /// Count array elements.
        Count(Count),
    }
);

/// Construct [Function] from [Call] implementors.
macro_rules! function_enum {
    (
        $(#[$($eattr:tt)*])*
        $nm:ident {$(
            $(#[$($vattr:tt)*])*
            $vnm:ident( $(#[$($vtyattr:tt)*])* $vty:ty)
        ),+ $(,)?}
    ) => {
        $( #[$($eattr)*] )*
        pub enum $nm {$(
            $( #[$($vattr)*] )*
            $vnm($( #[$($vtyattr)*] )*$vty),
        )*}

        const _: () = {
            #[derive(Debug, Clone)]
            pub enum Callable {$(
                $vnm(<$vty as ToCallable>::Call),
            )*}

            impl ToCallable for $nm {
                type Call = Callable;

                fn to_callable(&self) -> Self::Call {
                    match self {$(
                        Self::$vnm(value) => Callable::$vnm(<$vty as ToCallable>::to_callable(value)),
                    )*}
                }
            }

            impl Call for Callable {
                fn call(
                    &self,
                    input: $crate::array_expr::value_array::ValueArray
                ) -> ::syn::Result<$crate::array_expr::value_array::ValueArray> {
                    match self {$(
                        Self::$vnm(value) => <$vty as ToCallable>::Call::call(value, input),
                    )*}
                }
            }
        };

        $crate::to_tokens_enum!($nm { $( $vnm($vty) ),* });
        $crate::lookahead_parse_enum!($nm { $( $vnm($vty) ),* });
    };
}
use function_enum;

/// Define a function spec.
macro_rules! spec_impl {
    (
        $(#[$($eattr:tt)*])*
        $nm:ident {$(
            $(#[$($vattr:tt)*])*
            $vnm:ident( $(#[$($vtyattr:tt)*])* $vty:ty)
        ),+ $(,)?}
    ) => {
        $( #[$($eattr)*] )*
        pub enum $nm {$(
            $( #[$($vattr)*] )*
            $vnm($( #[$($vtyattr)*] )*$vty),
        )*}

        $crate::to_tokens_enum!($nm { $( $vnm($vty) ),* });
        $crate::lookahead_parse_enum!($nm { $( $vnm($vty) ),* });
    };
}
pub(crate) use spec_impl;
