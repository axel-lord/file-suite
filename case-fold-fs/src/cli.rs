//! [Cli] impl.

use ::std::{os::fd::AsFd, path::PathBuf, thread, time::Duration};

use ::clap::Parser;
use ::color_eyre::eyre::eyre;
use ::rusqlite::DatabaseName;
use ::rustix::fs::{Mode, OFlags};
use ::signal_hook::{
    consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM},
    iterator::Signals,
};

use crate::{Correction, Fs, action};

/// Mount a directory as case folded.
#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
pub struct Cli {
    /// Source directory.
    pub(crate) source: PathBuf,
    /// Mount point, will be same as source if not given.
    pub(crate) mountpoint: Option<PathBuf>,

    /// Dump internal database to specified file.
    #[arg(long)]
    pub(crate) dump: Option<PathBuf>,

    /// Do not destroy database contents on deletion.
    #[arg(long)]
    pub(crate) leak: bool,
}

impl ::file_suite_common::Run for Cli {
    type Error = ::color_eyre::Report;

    fn run(self) -> Result<(), Self::Error> {
        let Self {
            source,
            mountpoint,
            dump,
            leak,
        } = self;
        let signal_kinds = &[SIGHUP, SIGTERM, SIGINT, SIGQUIT];
        let mut signals = Signals::new(signal_kinds).map_err(|err| eyre!(err))?;
        let signals_handle = signals.handle();

        let mut signals = signals.forever();

        let root_dir = ::rustix::fs::open(
            &source,
            OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::RDONLY,
            Mode::empty(),
        )
        .map_err(|err| eyre!(err))?;

        let db_name = format!(
            "file:mem{:X}?mode=memory&cache=shared",
            ::rand::random::<i128>()
        );
        let (tx, rx) = ::std::sync::mpsc::channel();

        ::log::info!("db name = {db_name}");

        ::rayon::scope(|r| -> ::color_eyre::Result<()> {
            let connection = ::rusqlite::Connection::open(&db_name).map_err(|err| eyre!(err))?;
            let mut session = ::fuser::Session::new(
                Fs::new(root_dir.as_fd(), &connection, r, &tx)?.leak(leak),
                mountpoint.unwrap_or_else(|| source.clone()),
                &[],
            )
            .map_err(|err| eyre!(err))?;
            let mut unmounter = session.unmount_callable();

            thread::scope(|s| -> ::color_eyre::Result<()> {
                thread::Builder::new()
                    .name("case-fold-fs-signal-handler".into())
                    .spawn_scoped(s, || {
                        for signal in &mut signals {
                            ::log::info!("received signal {signal:?}");
                            break;
                        }

                        if let Err(err) = unmounter.unmount() {
                            ::log::error!("failed when unmounting, {err}");
                        }

                        for signal in &mut signals {
                            ::log::error!("received signal {signal:?}, terminating immediatly");
                            ::std::process::exit(1);
                        }
                    })
                    .map_err(|err| eyre!(err))?;

                fn correct(
                    rx: std::sync::mpsc::Receiver<Correction>,
                    db_name: &str,
                    _leak: bool,
                ) -> ::color_eyre::Result<()> {
                    let connection = ::rusqlite::Connection::open(&db_name)?;
                    let mut correct_rc = action::CorrectRc::new(&connection)?;
                    for correction in rx {
                        match correction {
                            Correction::Rc { ino } => {
                                correct_rc.perform(ino)?;
                            }
                            Correction::Clean => {}
                            Correction::Stop => break,
                        }
                    }
                    Ok(())
                }

                let correction = thread::Builder::new()
                    .name("case-fold-fs-correction-handler".into())
                    .spawn_scoped(s, || match correct(rx, &db_name, leak) {
                        Ok(_) => ::log::info!("closing correction thread"),
                        Err(err) => ::log::error!("correction error\n{err}"),
                    })
                    .map_err(|err| eyre!(err))?;

                let timer = thread::Builder::new()
                    .name("case-fold-fs-timer".into())
                    .spawn_scoped(s, || {
                        loop {
                            if tx.send(Correction::Clean).is_err() {
                                break;
                            }
                            thread::park_timeout(Duration::from_millis(500));
                        }
                    })
                    .map_err(|err| eyre!(err))?;

                let result = session.run();

                if let Some(dump) = dump {
                    ::std::fs::write(dump, &*connection.serialize(DatabaseName::Main)?)?;
                }

                Correction::Stop.send(&tx);
                _ = correction.join();

                timer.thread().unpark();
                _ = timer.join();

                signals_handle.close();
                result.map_err(|err| eyre!(err))
            })?;

            Ok(())
        })
    }
}
