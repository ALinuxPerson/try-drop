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
pub mod panic {
    use std::borrow::Cow;
    use crate::{Error, TryDropStrategy};

    #[cfg_attr(feature = "derives", derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash))]
    pub struct PanicDropStrategy {
        pub message: Cow<'static, str>,
    }

    impl PanicDropStrategy {
        pub const fn new() -> Self {
            Self::with_static_message("error occured when dropping an object")
        }

        pub fn with_message(message: impl Into<Cow<'static, str>>) -> Self {
            Self { message: message.into() }
        }

        pub const fn with_static_message(message: &'static str) -> Self {
            Self { message: Cow::Borrowed(message) }
        }

        pub const fn with_dynamic_message(message: String) -> Self {
            Self { message: Cow::Owned(message) }
        }
    }

    impl TryDropStrategy for PanicDropStrategy {
        fn handle_error(&self, error: Error) {
            Err(error).expect(&self.message)
        }
    }

    impl Default for PanicDropStrategy {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(feature = "ds-write")]
pub mod write {}
