use crate::TryDropStrategy;
use std::process;

/// A drop strategy that aborts the program if the drop fails.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct AbortDropStrategy;

impl TryDropStrategy for AbortDropStrategy {
    fn handle_error(&self, _error: crate::Error) {
        process::abort()
    }
}
