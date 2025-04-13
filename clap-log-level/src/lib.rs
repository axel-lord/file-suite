#![doc = include_str!("../README.md")]

use ::std::fmt::Display;

use ::clap::{ValueEnum, builder::PossibleValue};

/// Level newtype.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Level(pub ::log::Level);

impl Default for Level {
    fn default() -> Self {
        Self(if cfg!(debug_assertions) {
            ::log::Level::Trace
        } else {
            ::log::Level::Info
        })
    }
}

impl_value_enum!(Level(::log::Level), Trace, Debug, Info, Warn, Error);

/// LevelFilter newtype.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LevelFilter(pub ::log::LevelFilter);

impl Default for LevelFilter {
    fn default() -> Self {
        Self(if cfg!(debug_assertions) {
            ::log::LevelFilter::Trace
        } else {
            ::log::LevelFilter::Info
        })
    }
}

impl_value_enum!(
    LevelFilter(::log::LevelFilter),
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off
);

/// Implement Display and ValueEnum for LogLevel.
macro_rules! impl_value_enum {
    ($nm:ident($ty:ty), $($var:ident),*) => {
        paste::paste! {
        impl $nm {
            #[doc = concat!("Convert to internal [", stringify!($ty), "]")]
            pub const fn into_inner(self) -> $ty {
                self.0
            }
        }
        }

        impl ValueEnum for $nm {
            fn value_variants<'a>() -> &'a [Self] {
                &[$(
                    Self(<$ty> :: $var),
                )*]
            }

            fn to_possible_value(&self) -> Option<PossibleValue> {
                paste::paste! {
                Some(match self.0 {$(
                    $ty::$var => PossibleValue::new(stringify!($var)).aliases([
                        stringify!([< $var:lower >]), stringify!([< $var:upper >])
                    ]),
                )*})
                }
            }
        }

        impl Display for $nm {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self.0 {$(
                    <$ty>::$var => write!(f, "{}", stringify!($var) ),
                )*}
            }
        }

        impl From<$nm> for $ty {
            fn from(value: $nm) -> $ty {
                value.into_inner()
            }
        }

        impl From<$ty> for $nm {
            fn from(value: $ty) -> $nm {
                $nm(value)
            }
        }
    };
}
use impl_value_enum;
