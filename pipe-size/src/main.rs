use std::fmt::Write as _;
use std::io::Write;
use std::sync::{atomic::AtomicU64, mpsc::TryRecvError};

use bytesize::ByteSize;
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Simple Cli program that prints amount of bytes pipe through it to stderr
/// on a timer, stdin is piped to stout
struct Cli {
    /// How many bytes will pass through,
    /// percentage will be printed if provided
    #[arg(short, long)]
    size: Option<u64>,

    #[arg(long)]
    /// How long to sleep between prints
    sleep: Option<f64>,

    #[arg(short = 'r', long)]
    carriage_return: bool,
}

fn main() {
    let Cli {
        size: _,
        sleep,
        carriage_return,
    } = Cli::parse();
    let count = AtomicU64::new(0);
    let count = &count;
    let sleep = sleep.unwrap_or(0.1);
    let stderr = std::io::stderr();
    let stderr = &stderr;

    std::thread::scope(|s| {
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        s.spawn(move || {
            let sleep_time = std::time::Duration::from_secs_f64(sleep);
            let mut width = 0;
            let mut stderr = stderr.lock();
            let mut buf = String::new();
            while let Err(TryRecvError::Empty) = rx.try_recv() {
                buf.clear();
                _ = write!(
                    &mut buf,
                    "{}",
                    ByteSize(count.load(std::sync::atomic::Ordering::Relaxed)),
                );

                width = width.max(buf.len());

                let _ = write!(
                    stderr,
                    "{: <width$}{}",
                    buf,
                    if carriage_return { '\r' } else { '\n' },
                );

                std::thread::sleep(sleep_time);
            }
        });

        for _ in 0..4096 {
            let _ = count.fetch_add(1024, std::sync::atomic::Ordering::Relaxed);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        tx.send(()).unwrap();
    });
    if carriage_return {
        let mut stderr = stderr.lock();
        _ = writeln!(stderr);
    }
}
