#![doc(hidden)]

pub mod syn {
    #![doc(hidden)]

    pub use ::syn::{Error, Result};
}

pub use ::proc_macro2::{Span, TokenStream};
pub use ::quote::ToTokens;
pub use ::syn::{
    MacroDelimiter, braced, bracketed, custom_keyword, parenthesized,
    parse::{Lookahead1, Parse, ParseBuffer, ParseStream},
    token::{Brace, Bracket, Paren},
};
