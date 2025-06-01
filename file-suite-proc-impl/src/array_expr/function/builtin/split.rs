//! [SplitArgs] impl.

use ::file_suite_proc_lib::{FromArg, kw_kind};

use crate::array_expr::{
    from_values::{FromValues, ensure_single},
    function::{Call, ToCallable},
    storage::Storage,
    typed_value::TypedValue,
    value::Value,
    value_array::ValueArray,
};

/// Split by a string.
#[derive(Debug, Clone)]
pub struct SplitByCallable {
    /// String to split values by.
    by: String,
}

impl FromArg for SplitByCallable {
    type Factory = TypedValue;

    fn from_arg(by: String) -> Self {
        Self { by }
    }
}

impl Call for SplitByCallable {
    fn call(&self, array: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        Ok(array.split_by_str(&self.by))
    }
}

kw_kind!(
    /// Keyword specified split
    SplitArgs;
    /// Enum containing possible values for [SplitKw].
    #[expect(non_camel_case_types)]
    SplitKind {
        /// Split by PascalCase.
        pascal,
        /// Split by camelCase.
        camel,
        /// Split by dashes '-'.
        kebab,
        /// Split by underscores '_'.
        snake,
        /// Split by double colons '::'.
        path,
        /// Split by spaces ' '.
        space,
        /// Split by dots '.'.
        dot,
    }
);

impl FromValues for SplitKind {
    fn from_values(values: &[Value]) -> crate::Result<Self> {
        Ok(ensure_single(values)?.parse()?)
    }
}

impl ToCallable for SplitArgs {
    type Call = SplitKind;

    fn to_callable(&self) -> Self::Call {
        self.kind
    }
}

impl Call for SplitKind {
    fn call(&self, values: ValueArray, _: &mut Storage) -> crate::Result<ValueArray> {
        Ok(match self {
            SplitKind::camel => {
                let mut value_vec = Vec::with_capacity(values.len());
                for value in values {
                    let mut value_str = value.as_str();
                    while let Some(idx) = value_str.rfind(char::is_uppercase) {
                        let found;
                        (value_str, found) = value_str.split_at(idx);
                        value_vec.push(
                            Value::new(found.into())
                                .with_ty_of(&value)
                                .with_span_of(&value),
                        );
                    }
                    let content = String::from(value_str);
                    value_vec.push(value.with_content(content));
                }
                value_vec.reverse();
                value_vec.into()
            }
            SplitKind::pascal => {
                let mut value_vec = Vec::with_capacity(values.len());
                for value in values {
                    let mut value_str = value.as_str();
                    while let Some(idx) = value_str.rfind(char::is_uppercase) {
                        let found;
                        (value_str, found) = value_str.split_at(idx);
                        value_vec.push(
                            Value::new(found.into())
                                .with_ty_of(&value)
                                .with_span_of(&value),
                        );
                    }
                    // pascal expects value_str to be empty but handles it not being so
                    // anyways, whilst camel always adds the value_str value even if it is
                    // empty.
                    if !value_str.is_empty() {
                        // value.set_content(value_str.into());
                        let content = String::from(value_str);
                        value_vec.push(value.with_content(content))
                    };
                }
                value_vec.reverse();
                value_vec.into()
            }
            SplitKind::kebab => values.split_by_str("-"),
            SplitKind::snake => values.split_by_str("_"),
            SplitKind::path => values.split_by_str("::"),
            SplitKind::space => values.split_by_str(" "),
            SplitKind::dot => values.split_by_str("."),
        })
    }
}

#[cfg(test)]
mod test {
    #![allow(
        missing_docs,
        clippy::missing_docs_in_private_items,
        clippy::missing_panics_doc
    )]

    use crate::array_expr::test::assert_arr_expr;

    #[test]
    fn split_path() {
        assert_arr_expr!(
            { (!split::a::path) -> split(path).trim.ty(ident) },
            { split a path },
        );

        assert_arr_expr!(
            { splitCamelCase -> split(camel) },
            { split Camel Case },
        );

        assert_arr_expr!(
            { "split, by, comma" -> split_by(", ").ty(ident) },
            { split by comma },
        );
    }
}
