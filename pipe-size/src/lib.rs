#![doc = include_str!("../README.md")]

use ::bytesize::ByteSize;
use ::clap::Parser;
use ::file_suite_common::Run;
use ::std::{
    io::{self, ErrorKind, Read, Write},
    time::{Duration, Instant},
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
/// Simple Cli program that prints amount of bytes pipe through it to stderr
/// on a timer, stdin is piped to stout
pub struct Cli {
    /// How many bytes will pass through,
    /// percentage will be printed if provided.
    #[arg(short, long)]
    size: Option<u64>,

    /// Size of buffer used when piping stdin to stdout.
    #[arg(short, long, default_value_t = 4096)]
    bufsize: usize,

    /// How long to sleep between prints, in seconds.
    #[arg(long, default_value = "1")]
    sleep: f64,
}

impl Run for Cli {
    type Error = ::color_eyre::Report;

    fn run(self) -> Result<(), Self::Error> {
        let Self {
            size,
            sleep,
            bufsize,
        } = self;

        let mut buf = ::std::iter::repeat_n(0u8, bufsize).collect::<Box<[u8]>>();

        let duration = Duration::from_secs_f64(sleep);

        let mut stdin = io::stdin().lock();
        let mut stdout = io::stdout().lock();
        let mut checkpoint = Instant::now();
        let mut written = 0usize;

        loop {
            let count = match stdin.read(&mut buf) {
                Ok(count) => count,
                Err(err) => {
                    if !matches!(err.kind(), ErrorKind::Interrupted) {
                        continue;
                    }
                    return Err(err.into());
                }
            };

            if count == 0 {
                break;
            }

            written += count;
            stdout.write_all(&buf[..count])?;

            if Instant::now().duration_since(checkpoint) >= duration {
                if let Some(s) = size {
                    ::log::info!(
                        "{} of {} written",
                        ByteSize(u64::try_from(written)?),
                        ByteSize(s)
                    );
                } else {
                    ::log::info!("{} written", ByteSize(u64::try_from(written)?))
                }
                checkpoint = Instant::now();
            }
        }

        if let Some(s) = size {
            ::log::info!(
                "finished writing {} of {}",
                ByteSize(u64::try_from(written)?),
                ByteSize(s)
            );
        } else {
            ::log::info!("finished writing {}", ByteSize(u64::try_from(written)?),);
        }

        Ok(())
    }
}
