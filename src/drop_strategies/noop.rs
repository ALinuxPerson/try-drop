use crate::TryDropStrategy;

/// A drop strategy which does nothing if a drop error occurs.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct NoOpDropStrategy;

impl TryDropStrategy for NoOpDropStrategy {
    fn handle_error(&self, _error: crate::Error) {}
}
