use crate::FallibleTryDropStrategy;
use parking_lot::Mutex;
use std::io;
use std::io::Write;
use std::string::ToString;
use std::vec::Vec;

/// A drop strategy which writes the message of an error to a writer.
#[cfg_attr(feature = "derives", derive(Debug))]
pub struct WriteDropStrategy<W: Write> {
    /// The writer to write to.
    pub writer: Mutex<W>,

    /// Whether or not to append a newline to the end of the message.
    pub new_line: bool,

    /// The message to add at the beginning of the message.
    pub prelude: Option<Vec<u8>>,
}

impl<W: Write> WriteDropStrategy<W> {
    /// Creates a new [`WriteDropStrategy`] with the given writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer: Mutex::new(writer),
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

impl WriteDropStrategy<io::Stderr> {
    /// Write to standard error.
    pub fn stderr() -> Self {
        let mut this = Self::new(io::stderr());
        this.new_line(true);
        this
    }
}

impl WriteDropStrategy<io::Stdout> {
    /// Write to standard output.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use crate::drop_strategies::PanicDropStrategy;
    use crate::PureTryDrop;
    use crate::test_utils::{ErrorsOnDrop, Fallible};

    #[test]
    fn test_write_drop_strategy() {
        let mut writer = Cursor::new(Vec::new());
        let mut strategy = WriteDropStrategy::new(&mut writer);
        strategy.prelude("error: ");
        let errors = ErrorsOnDrop::<Fallible, _>::given(
            strategy,
            PanicDropStrategy::DEFAULT,
        ).adapt();
        drop(errors);
        assert_eq!(
            writer.into_inner(),
            b"error: this will always fail\n",
        )
    }
}
