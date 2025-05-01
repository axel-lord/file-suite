//! [InputSplit] impl.

use ::syn::{
    Ident, LitStr, Token, bracketed, custom_punctuation,
    ext::IdentExt,
    parenthesized,
    parse::{End, Parse, ParseStream},
    token::Bracket,
};

use crate::{
    kebab::kebab_inner,
    util::{AnyOf3, Either, kw_kind},
};

/// Kebab expression input.
#[derive(Debug, Default)]
pub struct KebabInput {
    /// Arguments of expression.
    pub args: Vec<Either<Ident, LitStr>>,
    /// How to split arguments.
    pub split: Option<(Bracket, Split)>,
    /// Optional trailing arrow.
    pub arrow: Option<Token![->]>,
}

impl KebabInput {
    /// Returns true if a trailing arrow was parsed.
    pub const fn has_arrow(&self) -> bool {
        self.arrow.is_some()
    }

    /// Get the parsed split if any.
    pub const fn split(&self) -> Option<SplitKind> {
        if let Some((_, split)) = &self.split {
            Some(split.kind)
        } else {
            None
        }
    }

    /// Get args split according to parsed [SplitKind] or default.
    pub fn split_args(&self) -> Vec<String> {
        self.split().unwrap_or_default().transform_args(
            self.args
                .iter()
                .map(|arg| match arg {
                    Either::A(id) => id.to_string(),
                    Either::B(lit) => lit.value(),
                })
                .collect(),
        )
    }
}

impl Parse for KebabInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut out = Self::default();
        let Self { args, split, arrow } = &mut out;

        custom_punctuation!(KebabExcl, --!);
        loop {
            let lookahead = input.lookahead1();

            if lookahead.peek(End) {
                break;
            }

            if lookahead.peek(Token![->]) {
                *arrow = Some(input.parse()?);
                break;
            } else if lookahead.peek(KebabExcl) || /* Workaround to give correct error message.*/
                    (input.peek(Token![-]) && input.peek2(Token![-]) && input.peek3(Token![!]))
            {
                <Token![-]>::parse(input)?;
                <Token![-]>::parse(input)?;
                <Token![!]>::parse(input)?;
                let content;
                parenthesized!(content in input);
                match kebab_inner(&content)? {
                    AnyOf3::A(a) => Either::A(a),
                    AnyOf3::B(b) => Either::B(b),
                    AnyOf3::C(c) => {
                        return Err(::syn::Error::new_spanned(
                            c,
                            "int output may not be used in nested expressions",
                        ));
                    }
                };
            } else if lookahead.peek(Ident) {
                args.push(Either::A(Ident::parse_any(input)?));
            } else if lookahead.peek(LitStr) {
                args.push(Either::B(input.parse()?));
            } else if lookahead.peek(Bracket) {
                let content;
                let bracket = bracketed!(content in input);

                if !content.is_empty() {
                    *split = Some((bracket, content.parse()?));

                    if !content.is_empty() {
                        return Err(
                            content.error("no more input expected for input split specification")
                        );
                    }
                }
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(out)
    }
}

kw_kind!(
    /// A Split that was parsed an as such has a span.
    Split
    /// How input values should be split
    SplitKind (Default) {
        /// Values should be split as they are given.
        [default]
        Split split,
        /// Values should be split by camelCase or PascalCase convention.
        Pascal pascal,
        /// Values should be split by camelCase convention.
        Camel camel,
        /// Values should be split by dashes.
        Kebab kebab,
        /// Values should be split by underscores.
        Snake snake,
        /// Values should be split by spaces.
        Space space,
        /// Values should be counted.
        Count count,
    }
);

impl SplitKind {
    /// Transform args given to input into desired form.
    pub fn transform_args(self, args: Vec<String>) -> Vec<String> {
        match self {
            SplitKind::Split => args,
            SplitKind::Pascal => args
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
            SplitKind::Camel => args
                .iter()
                .flat_map(|s| s.split(char::is_uppercase))
                .map(String::from)
                .collect(),
            SplitKind::Kebab => args
                .iter()
                .flat_map(|s| s.split('-'))
                .map(String::from)
                .collect(),
            SplitKind::Snake => args
                .iter()
                .flat_map(|s| s.split('_'))
                .map(String::from)
                .collect(),
            SplitKind::Space => args
                .iter()
                .flat_map(|s| s.split(' '))
                .map(String::from)
                .collect(),
            SplitKind::Count => Vec::from([args.len().to_string()]),
        }
    }
}
