//! Run derive impl.

use ::proc_macro2::TokenStream;
use ::quote::{ToTokens, format_ident, quote};
use ::syn::{
    Arm, Fields, Token,
    parse::{Parse, Parser},
    parse_quote,
};

/// Custom keywords
mod kw {
    use ::syn::custom_keyword;

    custom_keyword!(error);
}

/// Error type attribute.
#[derive(Debug)]
struct ErrorAttr {
    /// Error keyword.
    err: kw::error,
    /// Equality token '='.
    eq_token: Token![=],
    /// Type to use.
    ty: ::syn::Type,
}

impl ToTokens for ErrorAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { err, eq_token, ty } = self;
        err.to_tokens(tokens);
        eq_token.to_tokens(tokens);
        ty.to_tokens(tokens);
    }
}

impl Parse for ErrorAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            err: input.parse()?,
            eq_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

/// # Errors
/// If given data that is not an enum, or if the enum has an invalid layout.
pub fn derive_run(data: ::syn::DeriveInput) -> ::syn::Result<TokenStream> {
    let ::syn::DeriveInput {
        attrs,
        vis: _,
        ident,
        generics,
        data,
    } = data;

    let crate_path: ::syn::Path = parse_quote!(::file_suite_common);

    if let ::syn::Data::Union(data_union) = &data {
        return Err(::syn::Error::new_spanned(
            data_union.union_token,
            "Run may only be derived for single/no field structs and enums with single/no field variants",
        ));
    }

    let mut run_error = None::<ErrorAttr>;
    for attr in attrs {
        if attr.path().get_ident().is_some_and(|ident| ident == "run") {
            let list = attr.meta.require_list()?;
            let value = ErrorAttr::parse.parse2(list.tokens.clone())?;

            if run_error.is_some() {
                return Err(::syn::Error::new_spanned(
                    value,
                    "the #[run(error = _)] attribute should only be specified once",
                ));
            }

            run_error = Some(value)
        }
    }

    let mut run_error = run_error.map(|err| err.ty);
    let mut set_err = |fields: &Fields| {
        if run_error.is_none() {
            run_error = fields.iter().next().map(|field| {
                let ty = &field.ty;
                parse_quote! {
                    <#ty as #crate_path::Run>::Error
                }
            });
        }
    };

    let entropy: u16 = ::rand::random();
    let body = match data {
        ::syn::Data::Struct(data_struct) => {
            set_err(&data_struct.fields);
            let members = data_struct.fields.members();
            let types = data_struct.fields.iter().map(|field| &field.ty);
            quote! {
                #(
                <#types as #crate_path::Run>::run(self.#members)?;
                )*
                ::core::result::Result::Ok(())
            }
        }
        ::syn::Data::Enum(data_enum) => {
            let arms = data_enum.variants.iter().map(|variant| -> Arm {
                set_err(&variant.fields);
                let members = variant.fields.members();
                let types = variant.fields.iter().map(|field| &field.ty);
                let field_names = variant
                    .fields
                    .members()
                    .map(|member| format_ident!("__{}_{:03X}", member, entropy))
                    .collect::<Vec<_>>();
                let ident = &variant.ident;

                parse_quote! {
                    Self::#ident { #( #members: #field_names ),* } => {
                        #(
                        <#types as #crate_path::Run>::run(#field_names)?;
                        )*
                        ::core::result::Result::Ok(())
                    }
                }
            });

            quote! {
                match self {
                    #( #arms )*
                }
            }
        }
        ::syn::Data::Union(_) => unreachable!(),
    };

    let mut generics = generics;
    for tparam in generics.type_params_mut() {
        tparam.bounds.push(parse_quote!(#crate_path::Run));
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let err = run_error.unwrap_or_else(|| parse_quote!(::core::convert::Infallible));

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #crate_path::Run for #ident #ty_generics #where_clause {
            type Error = #err;

            #[allow(non_snake_case)]
            #[inline]
            fn run(self) -> ::core::result::Result<(), Self::Error> {
                #body
            }
        }
    })
}
