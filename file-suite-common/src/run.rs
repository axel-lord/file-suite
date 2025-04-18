//! [Run] definition.

/// Trait for types that may be ran by consuming an instance.
pub trait Run {
    /// Error used by cli.
    type Err;

    /// Run this cli.
    ///
    /// # Errors
    /// If the implementation whishes to.
    fn run(self) -> Result<(), Self::Err>;
}
