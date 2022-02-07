use crate::TryDropStrategy;
use std::process;

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
