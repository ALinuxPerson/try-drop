use std::io;
use std::io::Write;
use parking_lot::Mutex;
use try_drop::FallibleTryDropStrategy;

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