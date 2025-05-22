//! Macros to help with creating functions.

/// Construct Function from Call implementors.
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
            ) -> ::std::result::Result<$crate::array_expr::value_array::ValueArray, ::std::borrow::Cow<'static, str>> {
                match self {$(
                    Self::$vnm(value) => <$vty as ToCallable>::Call::call(value, input),
                )*}
            }
        }
    };

    $crate::to_tokens_enum!($nm { $( $vnm ),* });
    $crate::lookahead_parse_enum!($nm { $( $vnm($vty) ),* });
};
}
pub(crate) use function_enum;

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

    $crate::to_tokens_enum!($nm { $( $vnm ),* });
    $crate::lookahead_parse_enum!($nm { $( $vnm($vty) ),* });
};
}
pub(crate) use spec_impl;

/// Define syntax of a function.
macro_rules! function_struct {
(
    $(#[$($eattr:tt)*])*
    $nm:ident {$(
        $(#[$($fattr:tt)*])*
        $([$pattr:ident])? $fnm:ident: $fty:ty
    ),+ $(,)?}
) => {
    $( #[$($eattr)*] )*
    pub struct $nm {
        #[doc = "Function keyword"]
        pub kw: kw::$nm,
    $(
        $( #[$($fattr)*] )*
        pub $fnm: $fty,
    )*}

    $crate::lookahead_parse_keywords!($nm);
    $crate::to_tokens_struct!($nm {kw $(, $fnm)*});
    $crate::lookahead_parse_struct!($nm { kw: kw::$nm $(, $([$pattr])* $fnm: $fty )* });
};
}
pub(crate) use function_struct;
