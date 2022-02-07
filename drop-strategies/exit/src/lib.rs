use std::process;
use try_drop::TryDropStrategy;

pub struct ExitDropStrategy {
    pub exit_code: i32,
}

impl ExitDropStrategy {
    pub const DEFAULT: Self = Self::new(1);

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
    fn handle_error(&self, _error: try_drop::Error) {
        process::exit(self.exit_code)
    }
}
