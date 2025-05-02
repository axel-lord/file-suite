//! Anonymous sum types.

use ::std::any::Any;

use crate::any_conv::TypeInit;

crate::any_of!(Either: A B);

/// Trait used to convert between [any_of][crate::any_of!] created types.
pub trait AnyOfConv
where
    Self: Sized,
{
    /// Try to convert into another [any_of][crate::any_of!] created type T.
    ///
    /// # Errors
    /// If the conversion cannot be made the original value should be returned.
    fn try_any_of_conv<T>(self) -> Result<T, Self>
    where
        T: TypeInit + AsMut<dyn Any>;

    /// Convert into another [any_of][crate::any_of!] created type T.
    ///
    /// # Panics
    /// If the conversion is not possible.
    fn any_of_conv<T>(self) -> T
    where
        T: TypeInit + AsMut<dyn Any>,
    {
        self.try_any_of_conv()
            .unwrap_or_else(|_| panic!("could not convert between any_of types"))
    }
}

/// Generate an enum that may be one of a set of types.
/// Where `$nm` is the name of the type and `$ty` are the variant names.
/// # Example
///
/// ```
/// use ::file_suite_dyn::any_of;
/// any_of!(Either: L R);
/// ```
/// ```
/// pub enum Either<L, R> {
///     L(L),
///     R(R),
/// }
///
/// ```
/// ```
/// use ::file_suite_dyn::any_of;
/// any_of!(AnyOf2: A B);
/// ```
/// ```
/// pub enum AnyOf2<A, B> {
///     A(A),
///     B(B),
/// }
///
/// ```
/// ```
/// use ::file_suite_dyn::any_of;
/// any_of!(AnyOf3: A B C);
/// ```
/// ```
/// pub enum AnyOf3<A, B, C> {
///     A(A),
///     B(B),
///     C(C),
/// }
/// ```
#[macro_export]
macro_rules! any_of {
    ($nm:ident: $($ty:ident)+) => {
        $crate::kebab_paste! {

        #[doc = --!( "Value that may be one of " --!( $($ty)* -> str[count]) " types." -> str)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $nm<$($ty),*> {$(
            #[doc = --!( "Value has type " $ty "." -> str)]
            $ty($ty),
        )*}

        impl<$($ty: ::core::any::Any),*> $crate::any_of::AnyOfConv for $nm<$($ty),*> {
            fn try_any_of_conv<_T>(self) -> ::core::result::Result<_T, Self>
            where
                _T: $crate::any_conv::TypeInit + ::core::convert::AsMut<dyn ::core::any::Any>,
            {
                match self {$(
                    Self::$ty(mut value) => {
                        if let Some(value) = $crate::any_conv::TypeInit::init_from(&mut value) {
                            Ok(value)
                        } else {
                            Err(Self::$ty(value))
                        }
                    }
                )*}
            }
        }

        impl<$($ty: ::core::any::Any),*> ::core::convert::From<$nm<$($ty),*>> for $crate::Box<dyn ::core::any::Any> {
            fn from(value: $nm<$($ty),*>) -> Self {
                match value {$(
                    $nm::$ty(value) => {
                        let b: $crate::Box<dyn ::core::any::Any> = $crate::Box::new(value);
                        b
                    }
                )*}
            }
        }

        impl<$($ty: ::core::any::Any),*> ::core::convert::TryFrom<$crate::Box<dyn ::core::any::Any>> for $nm<$($ty),*> {
            type Error = $crate::Box<dyn ::core::any::Any>;

            fn try_from(mut value: $crate::Box<dyn ::core::any::Any>) -> ::core::result::Result<Self, Self::Error> {
                $(
                value = match value.downcast::<$ty>() {
                    ::core::result::Result::Err(any) => any,
                    ::core::result::Result::Ok(value) => return Ok(Self::$ty(*value)),
                };
                )*
                Err(value)
            }
        }

        impl<$($ty: ::core::any::Any + ::core::default::Default),*> $crate::any_conv::TypeInit for $nm<$($ty),*> {
            fn type_init(id: ::core::any::TypeId) -> Option<Self> {
                $(
                if id == ::core::any::TypeId::of::<$ty>() {
                    Some(Self::$ty(::core::default::Default::default()))
                } else
                )*
                {
                    None
                }
            }
        }

        impl<$($ty: ::core::any::Any),*> ::core::convert::AsMut<dyn ::core::any::Any> for $nm<$($ty),*> {
            fn as_mut(&mut self) -> &mut dyn ::core::any::Any {
                match self {$(
                    Self::$ty(mut_ref) => mut_ref,
                )*}
            }
        }

        impl<$($ty: ::core::any::Any),*> ::core::convert::AsRef<dyn ::core::any::Any> for $nm<$($ty),*> {
            fn as_ref(&self) -> &dyn ::core::any::Any {
                match self {$(
                    Self::$ty(refr) => refr,
                )*}
            }
        }

        }
    };
}
