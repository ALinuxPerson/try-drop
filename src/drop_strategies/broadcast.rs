//! Types and traits for the broadcast drop strategy.

mod private {
    pub trait Sealed {}
}

use crate::{FallibleTryDropStrategy, TryDropStrategy};
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;
use std::{fmt, io};
use tokio::runtime::Runtime;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::SendError;
use tokio::sync::broadcast::error::{RecvError, TryRecvError};
use tokio::sync::broadcast::{Receiver, Sender};

/// An async receiver, which is made sync via blocking on the tokio runtime.
#[cfg_attr(feature = "derives", derive(Debug))]
pub struct BlockingReceiver<T> {
    receiver: Receiver<T>,
    runtime: Arc<Runtime>,
}

impl<T: Clone> BlockingReceiver<T> {
    pub(crate) fn new(receiver: Receiver<T>, runtime: Arc<Runtime>) -> Self {
        Self { receiver, runtime }
    }

    /// Receive a message from the channel, blocking until one is available.
    pub fn recv(&mut self) -> Result<T, RecvError> {
        self.runtime.block_on(self.receiver.recv())
    }

    /// Try to receive a message from the channel, without blocking.
    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.receiver.try_recv()
    }
}

/// A wrapper against [`crate::Error`], implementing [`std::error::Error`].
#[derive(Debug, Clone)]
pub struct ArcError(pub Arc<crate::Error>);

impl ArcError {
    /// Create a new [`ArcError`] from a [`crate::Error`].
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

/// How to handle errors when sending a message to all receivers.
pub trait Mode: private::Sealed {}

/// Continue on sending errors to nobody if no receivers are available.
pub enum OkIfAlone {}

impl Mode for OkIfAlone {}

impl private::Sealed for OkIfAlone {}

/// Return an error if there are no receivers to send errors to.
pub enum NeedsReceivers {}

impl Mode for NeedsReceivers {}

impl private::Sealed for NeedsReceivers {}

/// A drop strategy which broadcasts a drop error to all receivers.
#[cfg_attr(feature = "derives", derive(Debug, Clone))]
pub struct BroadcastDropStrategy<M: Mode> {
    sender: Sender<ArcError>,
    runtime: Arc<Runtime>,
    _mode: PhantomData<M>,
}

impl<M: Mode> BroadcastDropStrategy<M> {
    /// Create a new broadcast drop strategy.
    #[cfg(feature = "ds-broadcast-new")]
    pub fn new(capacity: usize) -> io::Result<(Self, BlockingReceiver<ArcError>)> {
        Ok(Self::new_with(capacity, Runtime::new()?))
    }

    /// Create a new broadcast drop strategy, with a runtime.
    pub fn new_with(capacity: usize, runtime: Runtime) -> (Self, BlockingReceiver<ArcError>) {
        let (sender, receiver) = broadcast::channel(capacity);
        let runtime = Arc::new(runtime);
        let receiver = BlockingReceiver::new(receiver, Arc::clone(&runtime));

        (
            Self {
                sender,
                runtime,
                _mode: PhantomData,
            },
            receiver,
        )
    }

    /// Subscribe to this drop strategy, receiving new errors.
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
