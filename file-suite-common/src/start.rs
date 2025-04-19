//! [Start] definition.

use ::clap::Command;

mod sealed {
    //! [Sealed] trait definition.

    /// Sealed traits.
    pub trait Sealed {}

    impl<T> Sealed for crate::startable::Startable<T> {}
}

/// Trait for types that may be created and ran, intended for use as a dynamic trait using
/// [startable][crate::startable]
pub trait Start: sealed::Sealed {
    /// Attach [LogConfig][::log_level_cli::LogConfig], parse and call [run][crate::Run::run].
    ///
    /// # Errors
    /// If panic handler cannot be installed or the [Run::run][crate::Run::run] implementation needs to.
    fn start(&self, modules: &[&str]) -> crate::Result;

    /// Get command with [LogConfig][::log_level_cli::LogConfig] attached.
    fn standalone_command(&self) -> Command;
}
