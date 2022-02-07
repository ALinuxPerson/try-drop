use try_drop::{Error, TryDropStrategy};

pub struct NoOpDropStrategy;

impl TryDropStrategy for NoOpDropStrategy {
    fn handle_error(&self, _error: Error) {}
}