//! [Join] impl.

use ::quote::ToTokens;
use ::syn::{LitChar, LitStr, MacroDelimiter, parse::End};

use crate::{
    array_expr::{
        function::{Call, ToCallable, spec_impl},
        value_array::ValueArray,
    },
    util::{
        MacroDelimExt, ensure_empty, kw_kind, lookahead_parse::LookaheadParse, macro_delimited,
    },
};

#[doc(hidden)]
mod kw {
    use ::syn::custom_keyword;

    custom_keyword!(join);
}

kw_kind!(
    /// Keyword specified join.
    SpecKw;
    /// Enum of possible values for [SpecKw].
    #[expect(non_camel_case_types)]
    SpecKwKind: Default {
        #[default]
        concat,
        kebab,
        snake,
        path,
        space,
        dot,
    }
);

spec_impl!(
    /// Specification for how to join values.
    #[derive(Debug, Clone)]
    Spec {
        /// Join by string.
        Str(LitStr),
        /// Join by char.
        Char(LitChar),
        /// Join according to keyword.
        Kw(SpecKw),
    }
);

/// Join input.
#[derive(Debug, Clone)]
pub struct Join {
    /// Join keyword.
    kw: kw::join,
    /// Delim for spec.
    delim: Option<MacroDelimiter>,
    /// Specification for how to join values.
    spec: Option<Spec>,
}

/// [Call] implementor for [Join].
#[derive(Debug, Clone)]
pub enum JoinCallable {
    /// Join by a string.
    Str(String),
    /// Join by a char.
    Char(char),
    /// Join according to keyword.
    Kw(SpecKwKind),
}

impl Call for JoinCallable {
    fn call(&self, input: ValueArray) -> syn::Result<ValueArray> {
        Ok(match self {
            JoinCallable::Str(sep) => input.join_by_str(sep),
            JoinCallable::Char(sep) => {
                let mut buf = [0u8; 4];
                let sep = sep.encode_utf8(&mut buf) as &str;
                input.join_by_str(sep)
            }
            JoinCallable::Kw(kind) => match kind {
                SpecKwKind::concat => input.join_by_str(""),
                SpecKwKind::kebab => input.join_by_str("-"),
                SpecKwKind::snake => input.join_by_str("_"),
                SpecKwKind::path => input.join_by_str("::"),
                SpecKwKind::space => input.join_by_str(" "),
                SpecKwKind::dot => input.join_by_str("."),
            },
        })
    }
}

impl ToCallable for Join {
    type Call = JoinCallable;

    fn to_callable(&self) -> Self::Call {
        let Some(spec) = &self.spec else {
            return JoinCallable::Kw(SpecKwKind::concat);
        };

        match spec {
            Spec::Str(lit_str) => JoinCallable::Str(lit_str.value()),
            Spec::Char(lit_char) => JoinCallable::Char(lit_char.value()),
            Spec::Kw(spec_kw) => JoinCallable::Kw(spec_kw.kind),
        }
    }
}

impl LookaheadParse for Join {
    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        lookahead
            .peek(kw::join)
            .then(|| {
                let kw = input.parse()?;
                let mut delim = None;
                let mut spec = None;

                if MacroDelimiter::input_peek(input) {
                    let content;
                    delim = Some(macro_delimited!(content in input));

                    let lookahead = content.lookahead1();

                    if !lookahead.peek(End) {
                        spec = Spec::lookahead_parse(&content, &lookahead)?;

                        if spec.is_none() {
                            return Err(lookahead.error());
                        }

                        ensure_empty(&content)?;
                    }
                }

                Ok(Self { kw, delim, spec })
            })
            .transpose()
    }
}

impl ToTokens for Join {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { kw, delim, spec } = self;
        kw.to_tokens(tokens);
        if let Some(delim) = delim {
            delim.surround(tokens, |tokens| spec.to_tokens(tokens));
        }
    }
}
