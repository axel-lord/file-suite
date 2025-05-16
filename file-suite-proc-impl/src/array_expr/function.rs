//! [Function] impl.

use ::std::fmt::Debug;

use crate::{
    array_expr::function::{case::Case, join::Join, split::Split},
    value::Value,
};

mod split;

mod join;

mod case;

/// Trait for functions which may transform a vec of values.
pub trait Call {
    /// Transform the passed input.
    ///
    /// # Errors
    /// If input may not be transformed according to specification.
    fn call(&self, input: Vec<Value>) -> ::syn::Result<Vec<Value>>;
}

function_enum!(
    /// Enum collecting [Call] implementors.
    #[derive(Debug)]
    Function {
        /// Split input according to specification
        Split(Split),
        /// Join input according to specification.
        Join(Join),
        /// Case input according to specification.
        Case(Case),
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

        impl Call for $nm {
            fn call(&self, input: Vec<Value>) -> ::syn::Result<Vec<Value>> {
                match self {$(
                    Self::$vnm(value) => <$vty as Call>::call(value, input),
                )*}
            }
        }

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
