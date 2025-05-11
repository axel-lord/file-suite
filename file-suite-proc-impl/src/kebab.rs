//! [parse_kebab] implementation.
#![deny(clippy::todo)]

use ::proc_macro2::{Group, Span, TokenStream, TokenTree};
use ::quote::ToTokens;
use ::syn::parse::{ParseStream, Parser};

use crate::{
    kebab::{
        input::KebabInput,
        output::KebabOutput,
        value::{TypedValue, Value},
    },
    util::token_lookahead,
};

mod input;

mod output;

mod value;

mod split;

mod combine;

mod case;

/// Parse kebab macro input.
///
/// # Errors
/// If the input cannot be kebabed.
pub(super) fn parse_kebab(input: ParseStream) -> ::syn::Result<TokenStream> {
    kebab_inner(input).and_then(|e| {
        let mut tokens = TokenStream::default();

        for value in e {
            TypedValue::try_from(value)?.to_tokens(&mut tokens);
        }

        Ok(tokens)
    })
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

        for value in kebab_inner.parse2(group.stream())? {
            TypedValue::try_from(value)?.to_tokens(&mut out);
        }
    }

    Ok(out)
}

/// Inner kebab value.
///
/// # Errors
/// If given input cannot be parsed.
fn kebab_inner(input: ParseStream) -> ::syn::Result<Vec<Value>> {
    // let out_span = input.span();
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
        .or_else(|| kebab_input.default_ty())
        .unwrap_or_default();

    let mut joined = combine.join(case.apply(kebab_input.split_args()));
    for value in &mut joined {
        value.set_ty(ty);
    }

    Ok(joined)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::missing_panics_doc, missing_docs)]

    use ::quote::quote;
    use ::syn::parse::Parser;

    use super::*;

    #[test]
    fn kebab_concat() {
        let val = kebab_inner.parse2(quote! {A B C});
        assert_eq!(val.unwrap(), vec![Value::new_ident("ABC").ty_eq()]);

        let val = kebab_inner.parse2(quote! {"Hello" There "N" ice});
        assert_eq!(
            val.unwrap(),
            vec![Value::new_ident("HelloThereNice").ty_eq()]
        );

        let val = kebab_inner.parse2(quote! {"Value" Concat -> str});
        assert_eq!(val.unwrap(), vec![Value::new_litstr("ValueConcat").ty_eq()]);
    }

    #[test]
    fn kebab_nested() {
        let val = kebab_inner.parse2(quote! {A B -!(Hello There -> str[space]) -> str});
        assert_eq!(
            val.unwrap(),
            vec![Value::new_litstr("ABHello There").ty_eq()]
        );
    }
}
