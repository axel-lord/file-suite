//! Print sizes of some types in use by proc-macros.

use ::file_suite_proc_impl::{
    array_expr::{ArrayExpr, function::Function},
    typed_value::TypedValue,
    value::Value,
};

/// Print sizes and names of input.
macro_rules! print_size {
    ($($ty:ty),* $(,)?) => {{$(
        println!("{}: {}", stringify!($ty), size_of::<$ty>());
    )*}};
}

/// Entrypoint
fn main() {
    print_size!(Value, TypedValue, ArrayExpr, Function);
}
