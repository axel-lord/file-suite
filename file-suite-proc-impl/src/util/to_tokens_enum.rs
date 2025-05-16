//! [to_tokens_enum] impl.

/// Implement [::quote::ToTokens] for an enum of single field tuple variants.
#[macro_export]
macro_rules! to_tokens_enum {
    ($nm:ident { $( $vnm:ident($vty:ty)),+ $(,)?}) => {
        impl ::quote::ToTokens for $nm {
            fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
                match self {$(
                    Self::$vnm(value) => <$vty as ::quote::ToTokens>::to_tokens(value, tokens),
                )*}
            }
        }
    };
}
