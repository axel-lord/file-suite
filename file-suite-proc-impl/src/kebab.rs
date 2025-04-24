//! [parse_kebab] implementation.
use ::proc_macro2::{Group, Span, TokenStream, TokenTree};
use ::quote::ToTokens;
use ::syn::{
    Ident, LitStr, Token, bracketed, custom_punctuation,
    ext::IdentExt,
    parenthesized,
    parse::{End, ParseStream, Parser},
    token::Bracket,
};

use crate::{
    kw,
    util::{Either, token_lookahead},
};

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

/// How input values should be split
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
enum InputSplit {
    /// Values should be concatenated.
    None,
    /// Values should be split as they are given.
    #[default]
    Split,
    /// Values should be split by camelCase or PascalCase convention.
    Pascal,
    /// Values should be split by camelCase convention.
    Camel,
    /// Values should be split by dashes.
    Kebab,
    /// Values should be split by underscores.
    Snake,
    /// Values should be split by spaces.
    Space,
}

/// How output should be combined.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
enum OutputCombine {
    /// Values should be concatenated without any separator.
    #[default]
    Concat,
    /// Values should be joined by a dash,
    Kebab,
    /// Values should be joined by an underscore.
    Snake,
    /// Values should be joined by a space.
    Space,
}

impl OutputCombine {
    /// Join input arguments.
    fn join(self, values: Vec<String>) -> String {
        values.join(match self {
            OutputCombine::Concat => "",
            OutputCombine::Kebab => "-",
            OutputCombine::Snake => "_",
            OutputCombine::Space => " ",
        })
    }
}

/// How output case should be modified.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
enum OutputCase {
    /// Keep case as is.
    #[default]
    Keep,
    /// Use camelCase.
    Camel,
    /// Use PascalCase.
    Pascal,
    /// Use UPPERCASE.
    Upper,
    /// Use LOWERCASE.
    Lower,
}

fn titlecase(value: &str) -> String {
    let mut chars = value.chars();
    chars
        .next()
        .map(|first| first.to_uppercase())
        .into_iter()
        .flatten()
        .chain(chars.flat_map(char::to_lowercase))
        .collect()
}

impl OutputCase {
    /// Apply casing to a string.
    fn apply(self, mut values: Vec<String>) -> Vec<String> {
        match self {
            OutputCase::Keep => (),
            OutputCase::Camel => {
                let mut values = values.iter_mut();
                if let Some(first) = values.next() {
                    *first = first.to_lowercase();
                }
                for value in values {
                    *value = titlecase(value);
                }
            }
            OutputCase::Pascal => {
                for value in values.iter_mut() {
                    *value = titlecase(value);
                }
            }
            OutputCase::Upper => {
                for value in values.iter_mut() {
                    *value = value.to_uppercase();
                }
            }
            OutputCase::Lower => {
                for value in values.iter_mut() {
                    *value = value.to_lowercase();
                }
            }
        };
        values
    }
}

/// What the type of the output should be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
enum OutputType {
    /// Output an identifier.
    #[default]
    Ident,
    /// Output a string literal.
    LitStr,
}

/// Inner kebab value.
///
/// # Errors
/// If given input cannot be parsed.
fn kebab_inner(input: ParseStream) -> ::syn::Result<Either<LitStr, Ident>> {
    let out_span = input.span();
    let args = parse_input(input)?;
    let (ty, comb, case) = parse_output_pattern(input)?;

    if !input.is_empty() {
        return Err(input.error("no further macro input expected"));
    }

    let args = case.apply(args);
    let joined = comb.join(args);

    Ok(match ty {
        OutputType::Ident => Either::B(Ident::new(&joined, out_span)),
        OutputType::LitStr => Either::A(LitStr::new(&joined, out_span)),
    })
}

/// Parse output pattern of kebab.
///
/// # Errors
/// If given an invalid pattern.
fn parse_output_pattern(
    input: ParseStream,
) -> ::syn::Result<(OutputType, OutputCombine, OutputCase)> {
    if input.peek(End) {
        return Ok(Default::default());
    }

    let ty = if input.peek(kw::str) {
        let _: kw::str = input.parse()?;
        Some(OutputType::LitStr)
    } else if input.peek(kw::ident) {
        let _: kw::ident = input.parse()?;
        Some(OutputType::Ident)
    } else {
        None
    };

    let mut comb = None;
    let mut case = None;

    if input.peek(Bracket) {
        let content;
        bracketed!(content in input);

        loop {
            let lookahead = content.lookahead1();

            if lookahead.peek(End) {
                break;
            }

            if comb.is_some() && case.is_some() {
                return Err(content.error("no further input expected"));
            }

            if comb.is_none() {
                if lookahead.peek(kw::concat) {
                    let _: kw::concat = content.parse()?;
                    comb = Some(OutputCombine::Concat);
                    continue;
                }

                if lookahead.peek(kw::kebab) {
                    let _: kw::kebab = content.parse()?;
                    comb = Some(OutputCombine::Kebab);
                    continue;
                }

                if lookahead.peek(kw::snake) {
                    let _: kw::snake = content.parse()?;
                    comb = Some(OutputCombine::Snake);
                    continue;
                }

                if lookahead.peek(kw::space) {
                    let _: kw::space = content.parse()?;
                    comb = Some(OutputCombine::Space);
                    continue;
                }
            }

            if case.is_none() {
                if lookahead.peek(kw::keep) {
                    let _: kw::keep = content.parse()?;
                    case = Some(OutputCase::Keep);
                    continue;
                }

                if lookahead.peek(kw::camel) {
                    let _: kw::camel = content.parse()?;
                    case = Some(OutputCase::Camel);
                    continue;
                }

                if lookahead.peek(kw::pascal) {
                    let _: kw::pascal = content.parse()?;
                    case = Some(OutputCase::Pascal);
                    continue;
                }

                if lookahead.peek(kw::upper) {
                    let _: kw::upper = content.parse()?;
                    case = Some(OutputCase::Upper);
                    continue;
                }

                if lookahead.peek(kw::lower) {
                    let _: kw::lower = content.parse()?;
                    case = Some(OutputCase::Lower);
                    continue;
                }
            }

            return Err(lookahead.error());
        }
    }

    Ok((
        ty.unwrap_or_default(),
        comb.unwrap_or_default(),
        case.unwrap_or_default(),
    ))
}

/// Parse input portion of kebab macro.
///
/// # Errors
/// If given invalid input.
fn parse_input(input: ParseStream) -> ::syn::Result<Vec<String>> {
    let mut args = Vec::<String>::new();
    let mut split = InputSplit::default();
    loop {
        let lookahead = input.lookahead1();

        if lookahead.peek(End) {
            break;
        }

        if lookahead.peek(Token![->]) {
            let _: Token![->] = input.parse()?;
            break;
        }

        custom_punctuation!(KebabExcl, --!);
        _ = lookahead.peek(KebabExcl); // Workaround to give correct error message.
        if input.peek(Token![-]) && input.peek2(Token![-]) && input.peek3(Token![!]) {
            let _: (Token![-], Token![-], Token![!]) =
                (input.parse()?, input.parse()?, input.parse()?);
            let content;
            parenthesized!(content in input);

            match kebab_inner(&content)? {
                Either::A(a) => args.push(a.value()),
                Either::B(b) => args.push(b.to_string()),
            };

            continue;
        }

        if lookahead.peek(Ident) {
            args.push(input.call(Ident::parse_any)?.to_string());
            continue;
        }

        if lookahead.peek(LitStr) {
            args.push(input.parse::<LitStr>()?.value());
            continue;
        }

        if lookahead.peek(Bracket) {
            let content;
            bracketed!(content in input);

            let lookahead = content.lookahead1();

            if lookahead.peek(End) {
                continue;
            }

            'case_match: {
                if lookahead.peek(kw::none) {
                    let _: kw::none = content.parse()?;
                    split = InputSplit::None;
                    break 'case_match;
                }

                if lookahead.peek(kw::split) {
                    let _: kw::split = content.parse()?;
                    split = InputSplit::Split;
                    break 'case_match;
                }

                if lookahead.peek(kw::camel) {
                    let _: kw::camel = content.parse()?;
                    split = InputSplit::Camel;
                    break 'case_match;
                }

                if lookahead.peek(kw::pascal) {
                    let _: kw::pascal = content.parse()?;
                    split = InputSplit::Pascal;
                    break 'case_match;
                }

                if lookahead.peek(kw::kebab) {
                    let _: kw::kebab = content.parse()?;
                    split = InputSplit::Kebab;
                    break 'case_match;
                }

                if lookahead.peek(kw::snake) {
                    let _: kw::snake = content.parse()?;
                    split = InputSplit::Snake;
                    break 'case_match;
                }

                if lookahead.peek(kw::space) {
                    let _: kw::space = content.parse()?;
                    split = InputSplit::Space;
                    break 'case_match;
                }

                return Err(lookahead.error());
            }

            if !content.is_empty() {
                return Err(content.error("no more input expected for input split specification"));
            }
            continue;
        }

        return Err(lookahead.error());
    }

    Ok(match split {
        InputSplit::None => Vec::from([args.join("")]),
        InputSplit::Split => args,
        InputSplit::Pascal => args
            .iter()
            .flat_map(|s| {
                s.split(char::is_uppercase)
                    .skip(if s.starts_with(char::is_uppercase) {
                        1
                    } else {
                        0
                    })
            })
            .map(String::from)
            .collect(),
        InputSplit::Camel => args
            .iter()
            .flat_map(|s| s.split(char::is_uppercase))
            .map(String::from)
            .collect(),
        InputSplit::Kebab => args
            .iter()
            .flat_map(|s| s.split('-'))
            .map(String::from)
            .collect(),
        InputSplit::Snake => args
            .iter()
            .flat_map(|s| s.split('_'))
            .map(String::from)
            .collect(),
        InputSplit::Space => args
            .iter()
            .flat_map(|s| s.split(' '))
            .map(String::from)
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::missing_panics_doc, missing_docs)]

    use ::proc_macro2::Span;
    use ::quote::quote;
    use ::syn::parse::Parser;

    use super::*;

    fn id_exp(s: &str) -> Either<LitStr, Ident> {
        Either::B(Ident::new(s, Span::call_site()))
    }

    fn litstr_exp(s: &str) -> Either<LitStr, Ident> {
        Either::A(LitStr::new(s, Span::call_site()))
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
