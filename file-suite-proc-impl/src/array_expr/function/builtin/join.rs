//! [join] impl.

use ::syn::{LitChar, LitStr};

use crate::{
    array_expr::{
        function::{Call, ToCallable, function_struct, spec_impl},
        value_array::ValueArray,
    },
    util::{group_help::GroupOption, kw_kind, lookahead_parse::ParseWrap},
};

kw_kind!(
    /// Keyword specified join.
    SpecKw;
    /// Enum of possible values for [SpecKw].
    #[expect(non_camel_case_types)]
    JoinKind: Default {
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

function_struct!(
    /// Join input.
    #[derive(Debug, Clone)]
    #[expect(non_camel_case_types)]
    join {
        /// Specification for how to join values.
        [optional] spec: Option<GroupOption<ParseWrap<Spec>>>,
    }
);

// lookahead_parse_struct!(Join { kw: kw::join, [optional] spec: Option<GroupOption<ParseWrap<Spec>>> });

/// [Call] implementor for [Join].
#[derive(Debug, Clone)]
pub enum JoinCallable {
    /// Join by a string.
    Str(String),
    /// Join by a char.
    Char(char),
    /// Join according to keyword.
    Kw(JoinKind),
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
                JoinKind::concat => input.join_by_str(""),
                JoinKind::kebab => input.join_by_str("-"),
                JoinKind::snake => input.join_by_str("_"),
                JoinKind::path => input.join_by_str("::"),
                JoinKind::space => input.join_by_str(" "),
                JoinKind::dot => input.join_by_str("."),
            },
        })
    }
}

impl ToCallable for join {
    type Call = JoinCallable;

    fn to_callable(&self) -> Self::Call {
        let Some(spec) = self.spec.as_ref().and_then(|spec| spec.content.as_ref()) else {
            return JoinCallable::Kw(JoinKind::concat);
        };

        match &spec.0 {
            Spec::Str(lit_str) => JoinCallable::Str(lit_str.value()),
            Spec::Char(lit_char) => JoinCallable::Char(lit_char.value()),
            Spec::Kw(spec_kw) => JoinCallable::Kw(spec_kw.kind),
        }
    }
}
