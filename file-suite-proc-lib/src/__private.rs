#![doc(hidden)]

pub mod syn {
    #![doc(hidden)]

    pub use ::syn::{Error, Result};
}

pub use ::proc_macro2::{Span, TokenStream};
pub use ::quote::ToTokens;
pub use ::syn::{
    custom_keyword,
    parse::{Lookahead1, Parse, ParseStream},
};
