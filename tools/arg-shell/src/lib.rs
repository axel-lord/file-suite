#![doc = include_str!("../README.md")]

use ::chumsky::{IterParser, Parser};

pub use bytestr::ByteStr;

use crate::token::Token;

mod bytestr;
mod token;

/// command line interface for arg-shell.
#[derive(Debug, ::clap::Parser)]
pub struct Cli {
    /// String to test impl on.
    teststr: String,
}

impl ::file_suite_common::Run for Cli {
    type Error = ::std::convert::Infallible;

    fn run(self) -> Result<(), Self::Error> {
        let parser = Token::parser()
            .map_with(|o, e| (o, e.span()))
            .repeated()
            .collect::<Vec<_>>();

        println!("{:#?}", parser.parse(self.teststr.as_bytes()));

        Ok(())
    }
}
