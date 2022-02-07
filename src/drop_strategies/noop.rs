use crate::TryDropStrategy;

#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct NoOpDropStrategy;

impl TryDropStrategy for NoOpDropStrategy {
    fn handle_error(&self, _error: crate::Error) {}
}
