//! [Join] impl.

use ::quote::ToTokens;
use ::syn::{LitChar, LitStr, MacroDelimiter, parse::End};

use crate::{
    array_expr::function::{Call, spec_impl},
    util::{
        MacroDelimExt, ensure_empty, kw_kind, lookahead_parse::LookaheadParse, macro_delimited,
    },
    value::Value,
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

/// Join array of values by `sep`.
fn join_values(values: Vec<Value>, sep: &str) -> Vec<Value> {
    Vec::from([Value::join(values, |value| value.join(sep))])
}

impl Call for SpecKwKind {
    fn call(&self, values: Vec<crate::value::Value>) -> syn::Result<Vec<crate::value::Value>> {
        Ok(match self {
            SpecKwKind::concat => join_values(values, ""),
            SpecKwKind::kebab => join_values(values, "-"),
            SpecKwKind::snake => join_values(values, "_"),
            SpecKwKind::path => join_values(values, "::"),
            SpecKwKind::space => join_values(values, " "),
            SpecKwKind::dot => join_values(values, "."),
        })
    }
}

impl Call for Join {
    fn call(&self, input: Vec<crate::value::Value>) -> syn::Result<Vec<crate::value::Value>> {
        let Some(spec) = self.spec.as_ref() else {
            return SpecKwKind::default().call(input);
        };

        Ok(match spec {
            Spec::Str(lit_str) => join_values(input, &lit_str.value()),
            Spec::Char(lit_char) => {
                let sep = lit_char.value();
                let mut buf = [0u8; 4];
                let sep = sep.encode_utf8(&mut buf) as &str;

                join_values(input, sep)
            }
            Spec::Kw(spec_kw) => return spec_kw.kind.call(input),
        })
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
