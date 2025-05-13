//! [KebabInput] impl.

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    MacroDelimiter, Token, custom_punctuation,
    parse::{End, Parse, ParseStream},
};

use crate::{
    kebab::{kebab_inner, split::Split},
    util::{MacroDelimExt, macro_delimited},
    value::{TyKind, Value},
};

/// Kebab expression input.
#[derive(Debug, Default)]
pub struct KebabInput {
    /// Optional leading exclamation.
    pub excl: Option<Token![!]>,
    /// Arguments of expression.
    pub args: Vec<Value>,
    /// How to split arguments.
    pub split_group: Option<SplitGroup>,
    /// Optional trailing arrow.
    pub arrow: Option<Token![->]>,
}

impl KebabInput {
    /// Returns true if a trailing arrow was parsed.
    pub const fn has_arrow(&self) -> bool {
        self.arrow.is_some()
    }

    /// Get the parsed split if any.
    pub const fn split(&self) -> Option<&Split> {
        if let Some(SplitGroup {
            split: Some(split), ..
        }) = &self.split_group
        {
            Some(split)
        } else {
            None
        }
    }

    /// Suggest default output type.
    pub const fn default_ty(&self) -> Option<TyKind> {
        if self.excl.is_some() {
            Some(TyKind::int)
        } else {
            None
        }
    }

    /// Get args split according to parsed [SplitKind] or default.
    pub fn split_args(&self) -> Vec<Value> {
        Split::transform_args(self.split(), &self.args)
    }
}

impl Parse for KebabInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut out = Self::default();
        let Self {
            excl,
            args,
            split_group,
            arrow,
        } = &mut out;

        // Stringify contents.
        if input.peek(Token![!]) {
            let span = input.span();
            *excl = Some(input.parse()?);

            let mut value = Value::from(input.parse::<TokenStream>()?.to_string());
            value.set_span(span);

            args.push(value);

            return Ok(out);
        }

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
                macro_delimited!(content in input);
                for value in kebab_inner(&content)? {
                    args.push(value);
                }
            } else if let Some(value) = Value::lookahead_parse(input, &lookahead)? {
                args.push(value);
            } else if MacroDelimiter::peek(&lookahead) {
                *split_group = Some(input.parse()?);
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(out)
    }
}

/// Split specification of input.
#[derive(Debug)]
pub struct SplitGroup {
    /// Bracket deliminating group.
    pub delim: MacroDelimiter,
    /// Split in group.
    pub split: Option<Split>,
}

impl Parse for SplitGroup {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            delim: macro_delimited!(content in input),
            split: if content.is_empty() {
                None
            } else {
                let split = content.parse()?;

                if !content.is_empty() {
                    return Err(
                        content.error("no more input expected for input split specification")
                    );
                }

                Some(split)
            },
        })
    }
}

impl ToTokens for SplitGroup {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.delim
            .surround(tokens, |tokens| self.split.to_tokens(tokens));
    }
}
