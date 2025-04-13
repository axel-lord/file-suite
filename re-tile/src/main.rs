#![doc = include_str!("../README.md")]

use ::std::{
    io::{BufReader, Cursor, Read, Write},
    num::NonZero,
    sync::LazyLock,
};

use ::clap::{Parser, ValueEnum, ValueHint, builder::PossibleValue};
use ::color_eyre::{Section, eyre::eyre};
use ::image::{ImageFormat, ImageReader};
use ::tap::Pipe;

use crate::transform::Align;

mod logging;

mod completions;

mod transform;

/// Image format to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Format(ImageFormat);

impl ValueEnum for Format {
    fn value_variants<'a>() -> &'a [Self] {
        /// Possible format variants.
        static VARIANTS: LazyLock<Vec<Format>> = LazyLock::new(|| {
            ImageFormat::all()
                .filter_map(|f| {
                    if f.can_read() && f.can_write() {
                        Some(Format(f))
                    } else {
                        None
                    }
                })
                .collect()
        });
        &VARIANTS
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self.0.extensions_str() {
            [primary, other @ ..] => Some(PossibleValue::new(primary).aliases(other)),
            _ => None,
        }
    }
}

/// Parse a width hight tuple.
fn parse_dimensions(dim: &str) -> Result<[NonZero<u16>; 2], std::num::ParseIntError> {
    if let Some((w, h)) = dim.split_once('x') {
        Ok([w.parse()?, h.parse()?])
    } else {
        let d = dim.parse()?;
        Ok([d, d])
    }
}

/// Application to rebuild tilemaps.
#[derive(Parser)]
#[command(author, version, long_about = None)]
struct Cli {
    /// Input file.
    #[arg(long, short, default_value_t, conflicts_with = "completions", value_hint = ValueHint::FilePath)]
    input: patharg::InputArg,

    /// Output file.
    #[arg(long, short, default_value_t, value_hint = ValueHint::FilePath)]
    output: patharg::OutputArg,

    /// Dimensions of tiles in input [format: Width[xHeight]].
    #[arg(
        long,
        short,
        value_parser = parse_dimensions,
        value_hint = ValueHint::Other,
        required_unless_present = "completions"
    )]
    from: Option<[NonZero<u16>; 2]>,

    /// Dimensions of tiles in output [format: Width[xHeight]].
    #[arg(
        long,
        short,
        value_parser = parse_dimensions,
        value_hint = ValueHint::Other,
        required_unless_present = "completions"
    )]
    to: Option<[NonZero<u16>; 2]>,

    /// Horizontal alignment of output tiles.
    #[arg(long, short = 'x', value_enum, default_value_t)]
    align_x: Align,

    /// Vertical alignment of output tiles.
    #[arg(long, short = 'y', value_enum, default_value_t)]
    align_y: Align,

    /// Draw image index offset by specified value on image.
    #[arg(long)]
    index: bool,

    /// Offset to use for index.
    #[arg(
        long,
        default_value_t = 0,
        requires = "index",
        allow_negative_numbers = true
    )]
    offset: isize,

    /// Horizontal alignment of text.
    #[arg(
        long,
        visible_alias = "ty",
        value_enum,
        default_value_t,
        requires = "index"
    )]
    align_x_text: Align,

    /// Vertical alignment of text.
    #[arg(
        long,
        visible_alias = "tx",
        value_enum,
        default_value_t,
        requires = "index"
    )]
    align_y_text: Align,

    /// Format to use for output.
    #[arg(long, required_unless_present_any = ["output", "completions"])]
    format: Option<Format>,

    /// Logging setup.
    #[command(flatten)]
    logging: logging::Log,

    /// Completion setup.
    #[command(flatten)]
    completions: completions::Completions,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let Cli {
        logging,
        completions,
        output,
        input,
        from,
        to,
        format,
        align_x,
        align_y,
        index,
        offset,
        align_x_text,
        align_y_text,
    } = Cli::parse();
    logging.setup()?;

    if completions.completions {
        return completions.generate(output);
    }

    let from = from.expect("from should exist since completions is false");
    let to = to.expect("to should exist since completions is false");

    let mut buf = Vec::new();
    match input {
        ::patharg::InputArg::Stdin => std::io::stdin().lock().read_to_end(&mut buf)?,
        ::patharg::InputArg::Path(path) => std::fs::File::open(path)?
            .pipe(BufReader::new)
            .read_to_end(&mut buf)?,
    };

    let format = format
        .map(|format| format.0)
        .or_else(|| match &output {
            ::patharg::OutputArg::Stdout => None,
            ::patharg::OutputArg::Path(path) => ImageFormat::from_extension(path.extension()?),
        })
        .ok_or_else(|| {
            eyre!("could not deduce output format")
                .suggestion("use the --format option to set a format manually")
        })?;

    let image = transform::transform(
        ImageReader::new(Cursor::new(buf))
            .with_guessed_format()?
            .decode()?,
        from,
        to,
        align_x,
        align_y,
        index.then_some(offset),
        align_x_text,
        align_y_text,
    )?;

    let mut buf = Cursor::new(Vec::<u8>::new());
    image.write_to(&mut buf, format)?;

    output.create()?.write_all(&buf.into_inner())?;

    Ok(())
}
