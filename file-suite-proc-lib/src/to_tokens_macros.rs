//! Declarative macros for implementing [ToTokens][::quote::ToTokens].

/// Implement [::quote::ToTokens] for an enum of single field tuple variants.
#[macro_export]
macro_rules! to_tokens_enum {
    ($name:ident { $( $variant_name:ident ),* $(,)?}) => {
        impl ::quote::ToTokens for $name {
            fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
                match self {$(
                    Self::$variant_name(value) => value.to_tokens(tokens),
                )*}
            }
        }
    };
}

/// Implement [::quote::ToTokens] for a struct.
#[macro_export]
macro_rules! to_tokens_struct {
    ($name:ident { $( $field_name:ident),* $(,)?}) => {
        impl ::quote::ToTokens for $name {
            fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
                let Self { $($field_name),* } = self;
                $(
                $field_name.to_tokens(tokens);
                )*
            }
        }
    };
}
