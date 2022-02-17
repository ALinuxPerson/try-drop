use crate::FallibleTryDropStrategy;
use anyhow::Error;
use std::cell::RefCell;
use std::io;
use std::io::Write;
use std::string::ToString;
use std::vec::Vec;

/// A drop strategy which writes the message of an error to a writer. While more efficient than
/// it's thread safe counterpart, it's less flexible.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)
)]
pub struct ThreadUnsafeWriteDropStrategy<W: Write> {
    /// The writer to write to.
    pub writer: RefCell<W>,

    /// Whether or not to append a newline to the end of the message.
    pub new_line: bool,

    /// The message to add at the beginning of the message.
    pub prelude: Option<Vec<u8>>,
}

impl<W: Write> ThreadUnsafeWriteDropStrategy<W> {
    /// Creates a new [`ThreadUnsafeWriteDropStrategy`] with the given writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer: RefCell::new(writer),
            new_line: true,
            prelude: None,
        }
    }

    /// Sets whether or not to append a newline to the end of the message.
    pub fn new_line(&mut self, new_line: bool) -> &mut Self {
        self.new_line = new_line;
        self
    }

    /// Sets the message to add at the beginning of the message.
    pub fn prelude(&mut self, prelude: impl Into<Vec<u8>>) -> &mut Self {
        self.prelude = Some(prelude.into());
        self
    }
}

impl ThreadUnsafeWriteDropStrategy<io::Stderr> {
    /// Write to standard error.
    pub fn stderr() -> Self {
        Self::new(io::stderr())
    }
}

impl ThreadUnsafeWriteDropStrategy<io::Stdout> {
    /// Write to standard output.
    pub fn stdout() -> Self {
        Self::new(io::stdout())
    }
}

impl<W: Write> FallibleTryDropStrategy for ThreadUnsafeWriteDropStrategy<W> {
    type Error = io::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        let mut message = Vec::new();

        if let Some(prelude) = &self.prelude {
            message.extend_from_slice(prelude);
        }

        message.extend_from_slice(error.to_string().as_bytes());

        if self.new_line {
            message.push(b'\n')
        }

        self.writer.borrow_mut().write_all(&message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drop_strategies::PanicDropStrategy;
    use std::io::Cursor;
    use crate::test_utils::fallible_given;

    #[test]
    fn test_write_drop_strategy() {
        let mut writer = Cursor::new(Vec::new());
        let mut strategy = ThreadUnsafeWriteDropStrategy::new(&mut writer);
        strategy.prelude("error: ");
        drop(fallible_given(strategy, PanicDropStrategy::default()));
        assert_eq!(writer.into_inner(), b"error: this will always fail\n",)
    }
}

