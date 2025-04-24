//! Quick anonymous sum types.
#![allow(dead_code)]

use ::std::any::Any;

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;

any_of!(Either: A B);

/// Use [From] implementations betweed A, B and a boxed dyn [Any] to convert between to values.
pub fn any_conv<A, B>(value: A) -> B
where
    Box<dyn Any>: From<A>,
    B: From<Box<dyn Any>>,
{
    B::from(Box::<dyn Any>::from(value))
}

/// Use a [From] implementation to downcast [any_of] created enums to types.
///
/// # Errors
/// If the value cannot be gotten.
pub fn any_get<V, T>(container: T) -> Result<V, T>
where
    Box<dyn Any>: From<T>,
    T: From<Box<dyn Any>>,
    V: Any,
{
    let any = Box::<dyn Any>::from(container);

    any.downcast().map_err(T::from).map(|value| *value)
}

/// Use a [From] implementation to convert a value to an [any_of] enum.
pub fn from_any<V, T>(value: V) -> T
where
    T: From<Box<dyn Any>>,
    V: Any,
{
    T::from(Box::new(value))
}

/// Generate an enum that may be any of a set of types.
macro_rules! any_of {
($nm:ident: $($ty_nm:ident)+) => {
#[doc = "Value that may be one of many types."]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum $nm<$($ty_nm),*> {
    $(
    #[doc = concat!("Value has type ", stringify!($ty_nm))]
    $ty_nm($ty_nm),
    )*
}

impl<$($ty_nm: Any),*> From<$nm<$($ty_nm),*>> for Box<dyn Any> {
    fn from(value: $nm<$($ty_nm),*>) -> Box<dyn Any> {
        match value {
            $(
            $nm::$ty_nm(value) => {
                let b: Box<dyn Any> = Box::new(value);
                b
            }
            )*
        }
    }
}

impl<$($ty_nm: Any),*> From<Box<dyn Any>> for $nm<$($ty_nm),*> {
    fn from(mut value: Box<dyn Any>) -> Self {
        $(
        value = match value.downcast::<$ty_nm>() {
            Err(any) => any,
            Ok(val) => return Self::$ty_nm(*val),
        };
        )*
        _ = value;
        panic!("value was not of a compatible type")
    }
}

impl<$($ty_nm: ToTokens),*> ToTokens for $nm<$($ty_nm),*> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            $(
            Self::$ty_nm(val) => val.to_tokens(tokens),
            )*
        }
    }
}
};
}
use any_of;
