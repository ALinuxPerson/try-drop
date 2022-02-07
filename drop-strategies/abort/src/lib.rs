use std::process;
use try_drop::TryDropStrategy;

pub struct AbortDropStrategy;

impl TryDropStrategy for AbortDropStrategy {
    fn handle_error(&self, _error: try_drop::Error) {
        process::abort()
    }
}