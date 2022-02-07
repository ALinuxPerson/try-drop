#[cfg(feature = "ds-abort")]
pub mod abort {
    use std::process;
    use crate::TryDropStrategy;

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
    use crate::TryDropStrategy;

    #[cfg_attr(feature = "derives", derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash))]
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
    use crate::TryDropStrategy;

    #[cfg_attr(feature = "derives", derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default))]
    pub struct NoOpDropStrategy;

    impl TryDropStrategy for NoOpDropStrategy {
        fn handle_error(&self, _error: crate::Error) {}
    }
}

#[cfg(feature = "ds-panic")]
pub mod panic {
    use std::borrow::Cow;
    use std::string::String;
    use crate::{Error, TryDropStrategy};

    #[cfg_attr(feature = "derives", derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash))]
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
pub mod write {
    use std::io;
    use std::io::Write;
    use parking_lot::Mutex;
    use crate::FallibleTryDropStrategy;
    use std::vec::Vec;

    #[cfg_attr(feature = "derives", derive(Debug))]
    pub struct WriteDropStrategy<W: Write> {
        pub writer: Mutex<W>,
        pub new_line: bool,
        pub prelude: Option<Vec<u8>>,
    }

    impl<W: Write> WriteDropStrategy<W> {
        pub fn new(writer: W) -> Self {
            Self {
                writer: Mutex::new(writer),
                new_line: true,
                prelude: None,
            }
        }

        pub fn new_line(&mut self, new_line: bool) -> &mut Self {
            self.new_line = new_line;
            self
        }

        pub fn prelude(&mut self, prelude: impl Into<Vec<u8>>) -> &mut Self {
            self.prelude = Some(prelude.into());
            self
        }
    }

    impl WriteDropStrategy<io::Stderr> {
        pub fn stderr() -> Self {
            let mut this = Self::new(io::stderr());
            this.new_line(true);
            this
        }
    }

    impl WriteDropStrategy<io::Stdout> {
        pub fn stdout() -> Self {
            let mut this = Self::new(io::stdout());
            this.new_line(true);
            this
        }
    }

    impl<W: Write> FallibleTryDropStrategy for WriteDropStrategy<W> {
        type Error = io::Error;

        fn try_handle_error(&self, error: anyhow::Error) -> Result<(), Self::Error> {
            let mut message = Vec::new();

            if let Some(prelude) = &self.prelude {
                message.extend_from_slice(prelude);
            }

            message.extend_from_slice(error.to_string().as_bytes());

            if self.new_line {
                message.push(b'\n')
            }

            self.writer.lock().write_all(&message)
        }
    }
}
