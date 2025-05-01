# Kebab Expressions
The `kebab!` and `kebab_paste` macros can be used to evaluate so-called kebab expressions.

These expressions work with identifiers, strings and sometimes numbers, to concatenate, convert and
manipulate the case of strings and/or identifiers.

The format of an expression is `INPUT [SPLIT] -> OUTPUT_TY [COMBINE CASE]` where `INPUT` may be one or
more string literals or identifiers, `SPLIT` specifies how they should be split up further, `OUTPUT_TY` what kind of
output to create, `COMBINE` how the values should be combined and `CASE` what casing to apply.

An example expression might be `BufWriter[pascal] -> str[lower kebab]` which will split up the input according to PascalCase,
then convert the casing on the parts to lowercase, and finally combine them according to kebab-case and output the value as
a string literal, producing the result `"buf-writer"`;

`INPUT` may contain nested kebab-expressions using the same syntax as `kebab_paste!` to nest them, with the caveat that only
parentheses may be used to deliminate them. In this case the inner expressions are evaluated first and theit output is given
to the outer one as would any other input be.

When not specified several parts of the expression are given contextual defaults. A minimal expression would be just
`INPUT` which would be equivalent to `INPUT [split] -> ident[concat keep]`, when using a `COMBINE` that is not supported as
an ident such as `kebab` or `space`, `OUTPUT_TY` defaults to `str` instead of `ident`. This default is also used when an 
incompatible `SPLIT` is used such as `count`.

Since `INPUT` may only be identifiers or string literals any other `OUTPUT_TY` may not be used in nested expressions.

## Output Type Keywords
These are the values which may be used for `OUTPUT_TY`.

`ident`,
Output an identifier such as `BufWriter`.

`str`,
Output a string literal such as `"buf-writer"`.

`int`,
Output an unsuffixed integer literal such as `132`, this output may not be used for nested expressions.

`float`,
Output an unsuffixed floating point literal such as `5.3`, this output may not be used for nested expressions.

## Split keywords
These are the values which may be used for `SPLIT`.

