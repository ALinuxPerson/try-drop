#[cfg(feature = "ds-abort")]
pub mod abort {
    use std::process;

    #[cfg_attr(feature = "derives", derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default))]
    pub struct AbortDropStrategy;

    impl TryDropStrategy for AbortDropStrategy {
        fn handle_error(&self, _error: crate::Error) {
            process::abort()
        }
    }
}

#[cfg(feature = "ds-broadcast")]
pub mod broadcast {}

#[cfg(feature = "ds-exit")]
pub mod exit {
    use std::process;

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
        fn handle_error(&self, _error: crate::Error) {
            process::exit(self.exit_code)
        }
    }
}

#[cfg(feature = "ds-noop")]
pub mod noop {
    pub struct NoOpDropStrategy;

    impl TryDropStrategy for NoOpDropStrategy {
        fn handle_error(&self, _error: crate::Error) {}
    }
}

#[cfg(feature = "ds-panic")]
pub mod panic {}

#[cfg(feature = "ds-write")]
pub mod write {}
