#![doc = include_str!("../README.md")]

use ::std::io::Write;

use ::chumsky::{IterParser, Parser};

use ::termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
pub use bytestr::ByteStr;

use crate::{ast::Ast, token::Token, withspan::WithSpan};

pub mod alias;
pub mod ast;
pub mod bytestr;
pub mod exec;
pub mod smallvec;
pub mod token;
pub mod withspan;

/// command line interface for arg-shell.
#[derive(Debug, ::clap::Parser)]
pub struct Cli {
    /// String to test impl on.
    teststr: String,

    /// Print tokens.
    #[arg(long)]
    print_tokens: bool,

    /// Print ast.
    #[arg(long)]
    print_ast: bool,

    /// Print colorized expression.
    #[arg(long)]
    colorize: bool,
}

impl ::file_suite_common::Run for Cli {
    type Error = ::std::convert::Infallible;

    fn run(self) -> Result<(), Self::Error> {
        let Self {
            teststr,
            print_tokens,
            colorize,
            print_ast,
        } = self;
        let parser = Token::parser()
            .map_with(|o, e| WithSpan::from((o, e.span())))
            .repeated()
            .collect::<Vec<_>>();

        let values = parser.parse(teststr.as_bytes());

        if print_tokens {
            println!("{values:#?}");
        }

        for err in values.errors() {
            eprintln!("at  {}, {err}", err.span());
        }

        if let Some(values) = values.output() {
            if print_ast {
                let ast = Ast::parser().parse(values.as_slice());
                println!("{ast:#?}");
            }
            if colorize {
                let stdout = StandardStream::stdout(ColorChoice::Auto);
                let mut stdout = stdout.lock();
                for WithSpan { value: token, span } in values {
                    match token {
                        Token::LParen | Token::RParen => {
                            stdout.set_color(&ColorSpec::new().set_fg(Some(Color::Cyan)))
                        }
                        Token::Pipe => {
                            stdout.set_color(&ColorSpec::new().set_fg(Some(Color::Magenta)))
                        }
                        Token::String(..) => {
                            stdout.set_color(&ColorSpec::new().set_fg(Some(Color::Yellow)))
                        }
                        Token::FString(..) => {
                            stdout.set_color(&ColorSpec::new().set_fg(Some(Color::Green)))
                        }
                        Token::Comment(..) => stdout.set_color(&ColorSpec::new().set_bold(true)),
                        Token::Ident(..) | Token::Whitespace => {
                            stdout.set_color(&ColorSpec::new().set_reset(true))
                        }
                    }
                    .unwrap();
                    stdout
                        .write_all(&teststr.as_bytes()[span.start..span.end])
                        .unwrap()
                }
                println!();
            }
        }

        Ok(())
    }
}
