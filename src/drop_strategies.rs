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
pub mod broadcast {
    mod private {
        pub trait Sealed {}
    }

    use tokio::sync::broadcast::error::{RecvError, TryRecvError};
    use std::error::Error;
    use std::{fmt, io};
    use std::marker::PhantomData;
    use std::sync::Arc;
    use tokio::runtime::Runtime;
    use tokio::sync::broadcast;
    use tokio::sync::broadcast::{Receiver, Sender};
    use tokio::sync::broadcast::error::SendError;
    use crate::{FallibleTryDropStrategy, TryDropStrategy};

    #[cfg_attr(feature = "derives", derive(Debug))]
    pub struct BlockingReceiver<T> {
        receiver: Receiver<T>,
        runtime: Arc<Runtime>,
    }

    impl<T: Clone> BlockingReceiver<T> {
        pub(crate) fn new(receiver: Receiver<T>, runtime: Arc<Runtime>) -> Self {
            Self {
                receiver,
                runtime,
            }
        }

        pub fn recv(&mut self) -> Result<T, RecvError> {
            self.runtime.block_on(self.receiver.recv())
        }

        pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
            self.receiver.try_recv()
        }
    }

    #[derive(Debug, Clone)]
    pub struct ArcError(pub Arc<crate::Error>);

    impl ArcError {
        pub fn new(error: crate::Error) -> Self {
            ArcError(Arc::new(error))
        }
    }

    impl fmt::Display for ArcError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.0.fmt(f)
        }
    }

    impl Error for ArcError {}

    pub trait Mode: private::Sealed {}

    pub enum OkIfAlone {}

    impl Mode for OkIfAlone {}
    impl private::Sealed for OkIfAlone {}

    pub enum NeedsReceivers {}

    impl Mode for NeedsReceivers {}
    impl private::Sealed for NeedsReceivers {}

    #[cfg_attr(feature = "derives", derive(Debug, Clone))]
    pub struct BroadcastDropStrategy<M: Mode> {
        sender: Sender<ArcError>,
        runtime: Arc<Runtime>,
        _mode: PhantomData<M>,
    }

    impl<M: Mode> BroadcastDropStrategy<M> {
        #[cfg(feature = "ds-broadcast-new")]
        pub fn new(capacity: usize) -> io::Result<(Self, BlockingReceiver<ArcError>)> {
            Ok(Self::new_with(capacity, Runtime::new()?))
        }

        pub fn new_with(capacity: usize, runtime: Runtime) -> (Self, BlockingReceiver<ArcError>) {
            let (sender, receiver) = broadcast::channel(capacity);
            let runtime = Arc::new(runtime);
            let receiver = BlockingReceiver::new(receiver, Arc::clone(&runtime));

            (Self { sender, runtime, _mode: PhantomData }, receiver)
        }

        pub fn subscribe(&self) -> BlockingReceiver<ArcError> {
            BlockingReceiver::new(self.sender.subscribe(), Arc::clone(&self.runtime))
        }
    }

    impl TryDropStrategy for BroadcastDropStrategy<OkIfAlone> {
        fn handle_error(&self, error: crate::Error) {
            let _ = self.sender.send(ArcError::new(error));
        }
    }

    impl FallibleTryDropStrategy for BroadcastDropStrategy<NeedsReceivers> {
        type Error = SendError<ArcError>;

        fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
            self.sender.send(ArcError::new(error)).map(|_| ())
        }
    }
}

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
    use std::string::ToString;
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
