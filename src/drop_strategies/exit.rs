use crate::TryDropStrategy;
use std::process;

/// A drop strategy which exits the program with a specific exit code if the drop fails.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
pub struct ExitDropStrategy {
    /// The exit code to use if the drop fails.
    pub exit_code: i32,
}

impl ExitDropStrategy {
    /// The default exit drop strategy.
    pub const DEFAULT: Self = Self::new(1);

    /// Create a new exit drop strategy.
    pub const fn new(exit_code: i32) -> Self {
        Self { exit_code }
    }
}

impl Default for ExitDropStrategy {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl TryDropStrategy for ExitDropStrategy {
    fn handle_error(&self, _error: crate::Error) {
        process::exit(self.exit_code)
    }
}
