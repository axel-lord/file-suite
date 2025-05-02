//! [parse_kebab] implementation.
use ::proc_macro2::{Group, Span, TokenStream, TokenTree};
use ::quote::ToTokens;
use ::syn::{
    Ident, LitInt, LitStr,
    parse::{ParseStream, Parser},
};

use crate::{
    kebab::{
        input::KebabInput,
        output::{KebabOutput, TyKind},
    },
    util::{AnyOf3, token_lookahead},
};

mod input;

mod output;

/// Parse kebab macro input.
///
/// # Errors
/// If the input cannot be kebabed.
pub(super) fn parse_kebab(input: ParseStream) -> ::syn::Result<TokenStream> {
    kebab_inner(input).map(|e| e.to_token_stream())
}

/// Parse kebab input in the same manner as paste, with one enclosing macro call.
///
/// # Errors
/// If given invalid input.
pub(super) fn kebab_paste(input: TokenStream) -> ::syn::Result<TokenStream> {
    let mut it = input.into_iter();
    let mut it = token_lookahead::<3>(&mut it);
    let mut out = TokenStream::default();

    loop {
        let Some(next) = it.peek::<0>() else {
            break;
        };

        if matches!(next, TokenTree::Group(..)) {
            let Some(TokenTree::Group(group)) = it.next() else {
                unreachable!()
            };
            let mut new_group = Group::new(group.delimiter(), kebab_paste(group.stream())?);
            new_group.set_span(group.span());
            new_group.to_tokens(&mut out);
            continue;
        }

        use crate::util::tcmp::pseq;
        if !it.matches(pseq!('-', '-', '!')) {
            it.next()
                .unwrap_or_else(|| unreachable!())
                .to_tokens(&mut out);
            continue;
        }

        let Some(last) = it.discard().next_back() else {
            unreachable!();
        };

        let err = |span: Span| ::syn::Error::new(span, "expected delimited group following '-!'");
        let tree = it.next().ok_or_else(|| err(last.span()))?;
        let TokenTree::Group(group) = tree else {
            return Err(err(tree.span()));
        };

        kebab_inner.parse2(group.stream())?.to_tokens(&mut out);
    }

    Ok(out)
}

/// Inner kebab value.
///
/// # Errors
/// If given input cannot be parsed.
fn kebab_inner(input: ParseStream) -> ::syn::Result<AnyOf3<LitStr, Ident, LitInt>> {
    let out_span = input.span();
    // let (args, arrow) = parse_input(input)?;
    let kebab_input = input.parse::<KebabInput>()?;

    let kebab_output = kebab_input
        .has_arrow()
        .then(|| input.parse::<KebabOutput>())
        .transpose()?
        .unwrap_or_default();

    if !input.is_empty() {
        return Err(input.error("no further macro input expected"));
    }

    let combine = kebab_output.combine().unwrap_or_default();
    let case = kebab_output.case().unwrap_or_default();
    let ty = kebab_output
        .ty()
        .or_else(|| combine.default_ty())
        .unwrap_or_default();

    let joined = combine.join(case.apply(kebab_input.split_args()));

    Ok(match ty {
        TyKind::Ident => AnyOf3::B(Ident::new(&joined, out_span)),
        TyKind::LitStr => AnyOf3::A(LitStr::new(&joined, out_span)),
        TyKind::LitInt => AnyOf3::C(LitInt::new(&joined, out_span)),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::missing_panics_doc, missing_docs)]

    use ::proc_macro2::Span;
    use ::quote::quote;
    use ::syn::parse::Parser;

    use super::*;

    fn id_exp(s: &str) -> AnyOf3<LitStr, Ident, LitInt> {
        AnyOf3::B(Ident::new(s, Span::call_site()))
    }

    fn litstr_exp(s: &str) -> AnyOf3<LitStr, Ident, LitInt> {
        AnyOf3::A(LitStr::new(s, Span::call_site()))
    }

    #[test]
    fn kebab_concat() {
        let val = kebab_inner.parse2(quote! {A B C});
        assert_eq!(val.unwrap(), id_exp("ABC"));

        let val = kebab_inner.parse2(quote! {"Hello" There "N" ice});
        assert_eq!(val.unwrap(), id_exp("HelloThereNice"));

        let val = kebab_inner.parse2(quote! {"Value" Concat -> str});
        assert_eq!(val.unwrap(), litstr_exp("ValueConcat"));
    }

    #[test]
    fn kebab_nested() {
        let val = kebab_inner.parse2(quote! {A B -!(Hello There -> str[space]) -> str});
        assert_eq!(val.unwrap(), litstr_exp("ABHello There"));
    }
}
