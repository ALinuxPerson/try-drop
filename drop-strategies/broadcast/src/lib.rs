mod receiver;
mod private {
    pub trait Sealed {}
}

use std::error::Error;
use std::{fmt, io};
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::broadcast::error::SendError;
use receiver::BlockingReceiver;
use try_drop::{FallibleTryDropStrategy, TryDropStrategy};

#[derive(Debug, Clone)]
pub struct ArcTryDropError(pub Arc<try_drop::Error>);

impl ArcTryDropError {
    pub fn new(error: try_drop::Error) -> Self {
        ArcTryDropError(Arc::new(error))
    }
}

impl fmt::Display for ArcTryDropError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for ArcTryDropError {}

pub trait Mode: private::Sealed {}

pub enum OkIfAlone {}

impl Mode for OkIfAlone {}
impl private::Sealed for OkIfAlone {}

pub enum NeedsReceivers {}

impl Mode for NeedsReceivers {}
impl private::Sealed for NeedsReceivers {}

pub struct BroadcastDropStrategy<M: Mode> {
    sender: Sender<ArcTryDropError>,
    runtime: Arc<Runtime>,
    _mode: PhantomData<M>,
}

impl<M: Mode> BroadcastDropStrategy<M> {
    #[cfg(feature = "new")]
    pub fn new(capacity: usize) -> io::Result<(Self, BlockingReceiver<ArcTryDropError>)> {
        Ok(Self::new_with(capacity, Runtime::new()?))
    }

    pub fn new_with(capacity: usize, runtime: Runtime) -> (Self, BlockingReceiver<ArcTryDropError>) {
        let (sender, receiver) = broadcast::channel(capacity);
        let runtime = Arc::new(runtime);
        let receiver = BlockingReceiver::new(receiver, Arc::clone(&runtime));

        (Self { sender, runtime, _mode: PhantomData }, receiver)
    }

    pub fn subscribe(&self) -> BlockingReceiver<ArcTryDropError> {
        BlockingReceiver::new(self.sender.subscribe(), Arc::clone(&self.runtime))
    }
}

impl TryDropStrategy for BroadcastDropStrategy<OkIfAlone> {
    fn handle_error(&self, error: try_drop::Error) {
        let _ = self.sender.send(ArcTryDropError::new(error));
    }
}

impl FallibleTryDropStrategy for BroadcastDropStrategy<NeedsReceivers> {
    type Error = SendError<ArcTryDropError>;

    fn try_handle_error(&self, error: try_drop::Error) -> Result<(), Self::Error> {
        self.sender.send(ArcTryDropError::new(error)).map(|_| ())
    }
}
