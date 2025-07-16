//! [Cli] impl.

use ::std::{os::fd::AsFd, path::PathBuf, thread};

use ::clap::Parser;
use ::color_eyre::eyre::eyre;
use ::dashmap::DashMap;
use ::rusqlite::DatabaseName;
use ::rustix::fs::{Mode, OFlags};
use ::signal_hook::{
    consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM},
    iterator::Signals,
};

use crate::Fs;

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
        let file_descriptors = DashMap::default();

        ::rayon::scope(|r| -> ::color_eyre::Result<()> {
            let connection = ::rusqlite::Connection::open_in_memory().map_err(|err| eyre!(err))?;
            let mut session = ::fuser::Session::new(
                Fs::new(root_dir.as_fd(), &connection, r, &file_descriptors)?.leak(leak),
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

                let result = session.run();
                drop(session);

                if let Some(dump) = dump {
                    ::log::info!("dumping database to {dump:?}");
                    ::std::fs::write(dump, &*connection.serialize(DatabaseName::Main)?)?;
                }

                signals_handle.close();
                result.map_err(|err| eyre!(err))
            })?;

            Ok(())
        })
    }
}
