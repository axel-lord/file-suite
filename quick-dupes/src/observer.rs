use ::std::{
    ops::ControlFlow::{self, Break, Continue},
    sync::{Arc, atomic::Ordering::SeqCst},
    thread,
    time::Duration,
};

use ::color_eyre::{Section, eyre::eyre};

use crate::Shared;

pub struct Observer {
    shared: Arc<Shared>,
    current: usize,
}

impl Observer {
    pub const fn new(shared: Arc<Shared>) -> Self {
        Self { shared, current: 0 }
    }

    pub fn spawn(self) -> Result<thread::JoinHandle<()>, ::color_eyre::Report> {
        thread::Builder::new()
            .name("quick-dupes-observer".into())
            .spawn(self.main_loop())
            .map_err(|err| eyre!("failed to spawn observer thread").error(err))
    }

    fn main_loop(mut self) -> impl FnOnce() {
        move || loop {
            if self.iteration().is_break() {
                break;
            }
            thread::park_timeout(Duration::from_millis(500));
        }
    }

    fn info(&self, stage: usize) -> ControlFlow<()> {
        match stage {
            0 => ::log::info!(
                "walking directories, {} paths found",
                self.shared.total_paths.load(SeqCst)
            ),
            1 => {
                let total = self.shared.total_paths.load(SeqCst);
                let filtered = self.shared.filtered_paths.load(SeqCst);
                let percentage = (filtered as f64 / total as f64) * 100.0;
                ::log::info!("filtering paths {filtered}/{total} ({percentage:.1}%)",)
            }
            2 => {
                let total = self.shared.total_paths.load(SeqCst);
                let hashed = self.shared.hashed_paths.load(SeqCst);
                let percentage = (hashed as f64 / total as f64) * 100.0;
                ::log::info!("hashing paths {hashed}/{total} ({percentage:.1}%)",)
            }
            _ => return Break(()),
        }
        Continue(())
    }

    fn iteration(&mut self) -> ControlFlow<()> {
        let status = self.shared.status.load(SeqCst);

        if status != self.current {
            self.info(self.current)?;
            self.current = status;
        }

        self.info(self.current)?;

        if Arc::get_mut(&mut self.shared).is_some() {
            Break(())
        } else {
            Continue(())
        }
    }
}
