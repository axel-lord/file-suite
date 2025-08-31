#![doc = include_str!("../README.md")]

use ::std::io::Write;

use ::chumsky::{IterParser, Parser};

use ::termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
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

        let values = parser.parse(self.teststr.as_bytes());

        println!("{values:#?}");

        if let Ok(values) = values.into_result() {
            let stdout = StandardStream::stdout(ColorChoice::Auto);
            let mut stdout = stdout.lock();
            for (token, span) in values {
                match token {
                    Token::LParen | Token::RParen => {
                        stdout.set_color(&ColorSpec::new().set_fg(Some(Color::Cyan)))
                    }
                    Token::Pipe => stdout.set_color(&ColorSpec::new().set_fg(Some(Color::Magenta))),
                    Token::String(..) => {
                        stdout.set_color(&ColorSpec::new().set_fg(Some(Color::Yellow)))
                    }
                    Token::FString(..) => {
                        stdout.set_color(&ColorSpec::new().set_fg(Some(Color::Green)))
                    }
                    Token::Comment(..) => stdout.set_color(&ColorSpec::new().set_bold(true)),
                    Token::Ident(..) | Token::Term | Token::Whitespace => {
                        stdout.set_color(&ColorSpec::new().set_reset(true))
                    }
                }
                .unwrap();
                stdout
                    .write_all(&self.teststr.as_bytes()[span.start..span.end])
                    .unwrap()
            }
            println!();
        }

        Ok(())
    }
}
