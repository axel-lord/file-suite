//! [to_tokens_enum] impl.

/// Implement [::quote::ToTokens] for an enum of single field tuple variants.
#[macro_export]
macro_rules! to_tokens_enum {
    ($nm:ident { $( $vnm:ident ),* $(,)?}) => {
        impl ::quote::ToTokens for $nm {
            fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
                match self {$(
                    Self::$vnm(value) => value.to_tokens(tokens),
                )*}
            }
        }
    };
}

/// Implement [::quote::ToTokens] for a struct.
#[macro_export]
macro_rules! to_tokens_struct {
    ($nm:ident { $( $vnm:ident),* $(,)?}) => {
        impl ::quote::ToTokens for $nm {
            fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
                let Self { $($vnm),* } = self;
                $(
                $vnm.to_tokens(tokens);
                )*
            }
        }
    };
}
