//! [Count] impl.

use ::quote::ToTokens;
use ::syn::MacroDelimiter;

use crate::{
    array_expr::{function::Call, value_array::ValueArray},
    util::{MacroDelimExt, ensure_empty, lookahead_parse::LookaheadParse, macro_delimited},
    value::{TyKind, Value},
};

#[doc(hidden)]
mod kw {
    use ::syn::custom_keyword;

    custom_keyword!(count);
}

/// Count amount of values passed.
#[derive(Debug, Clone)]
pub struct Count {
    /// Count keyword.
    kw: kw::count,
    /// Optional macro delimiter.
    delim: Option<MacroDelimiter>,
}

impl Call for Count {
    fn call(
        &self,
        input: crate::array_expr::value_array::ValueArray,
    ) -> syn::Result<crate::array_expr::value_array::ValueArray> {
        let mut value = Value::new_int(
            input
                .len()
                .try_into()
                .unwrap_or_else(|_| unreachable!("all lengths should fit in an isize")),
        );
        if let Some(span) = input.span() {
            value.set_span(span);
        }
        value.set_ty(TyKind::int);
        Ok(ValueArray::from_value(value))
    }
}

impl LookaheadParse for Count {
    fn lookahead_parse(
        input: syn::parse::ParseStream,
        lookahead: &syn::parse::Lookahead1,
    ) -> syn::Result<Option<Self>> {
        Ok(if lookahead.peek(kw::count) {
            let kw = input.parse()?;

            let mut delim = None;
            if MacroDelimiter::input_peek(input) {
                let content;
                delim = Some(macro_delimited!(content in input));
                ensure_empty(&content)?
            }

            Some(Self { kw, delim })
        } else {
            None
        })
    }
}

impl ToTokens for Count {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { kw, delim } = self;
        kw.to_tokens(tokens);
        if let Some(delim) = delim {
            delim.surround(tokens, |_| ());
        }
    }
}
