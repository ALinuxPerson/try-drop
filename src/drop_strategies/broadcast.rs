//! Types and traits for the broadcast drop strategy. This is built on top of the tokio broadcast
//! channel.

mod private {
    pub trait Sealed {}
}

use crate::{FallibleTryDropStrategy, TryDropStrategy};

use std::marker::PhantomData;

use crate::adapters::ArcError;
pub use tokio::runtime::Handle;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::SendError;
use tokio::sync::broadcast::error::{RecvError, TryRecvError};
pub use tokio::sync::broadcast::Receiver as AsyncReceiver;
use tokio::sync::broadcast::{Receiver, Sender};

/// An async receiver, which is made sync via blocking on a handle to the tokio runtime.
#[cfg_attr(feature = "derives", derive(Debug))]
pub struct BlockingReceiver<T> {
    receiver: Receiver<T>,
    handle: Handle,
}

impl<T: Clone> BlockingReceiver<T> {
    pub(crate) fn new(receiver: Receiver<T>, handle: Handle) -> Self {
        Self { receiver, handle }
    }

    /// Receive a message from the channel, blocking until one is available.
    pub fn recv(&mut self) -> Result<T, RecvError> {
        self.handle.block_on(self.receiver.recv())
    }

    /// Try to receive a message from the channel, without blocking.
    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.receiver.try_recv()
    }
}

/// How to handle errors when sending a message to all receivers.
pub trait Mode: private::Sealed {}

/// Continue on sending errors to nobody if no receivers are available.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
pub enum OkIfAlone {}

impl Mode for OkIfAlone {}

impl private::Sealed for OkIfAlone {}

/// Return an error if there are no receivers to send errors to.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
pub enum NeedsReceivers {}

impl Mode for NeedsReceivers {}

impl private::Sealed for NeedsReceivers {}

/// A drop strategy which broadcasts a drop error to all receivers.
#[cfg_attr(feature = "derives", derive(Debug, Clone))]
pub struct BroadcastDropStrategy<M: Mode> {
    sender: Sender<ArcError>,
    handle: Handle,
    _mode: PhantomData<M>,
}

impl<M: Mode> BroadcastDropStrategy<M> {
    /// Create a new broadcast drop strategy from a handle to the current tokio runtime.
    pub fn new(capacity: usize) -> (Self, BlockingReceiver<ArcError>) {
        Self::new_with(capacity, Handle::current())
    }

    /// Create a new broadcast drop strategy, with a handle to a tokio runtime.
    pub fn new_with(capacity: usize, handle: Handle) -> (Self, BlockingReceiver<ArcError>) {
        let (sender, receiver) = broadcast::channel(capacity);
        let receiver = BlockingReceiver::new(receiver, handle.clone());

        (
            Self {
                sender,
                handle,
                _mode: PhantomData,
            },
            receiver,
        )
    }

    /// Subscribe to this drop strategy, receiving new errors.
    pub fn subscribe(&self) -> BlockingReceiver<ArcError> {
        BlockingReceiver::new(self.sender.subscribe(), self.handle.clone())
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
