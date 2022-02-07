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
