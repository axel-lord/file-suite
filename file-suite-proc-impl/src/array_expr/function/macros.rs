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
                input: $crate::array_expr::value_array::ValueArray,
                storage: &mut $crate::array_expr::storage::Storage,
            ) -> $crate::Result<$crate::array_expr::value_array::ValueArray> {
                match self {$(
                    Self::$vnm(value) => <$vty as ToCallable>::Call::call(value, input, storage),
                )*}
            }
        }
    };

    ::file_suite_proc_lib::to_tokens_enum!($nm { $( $vnm ),* });
    ::file_suite_proc_lib::lookahead_parse_enum!($nm {$($vnm: $vty),*});
};
}
pub(crate) use function_enum;
