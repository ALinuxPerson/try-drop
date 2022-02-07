use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::broadcast::error::{RecvError, TryRecvError};
use tokio::sync::broadcast::Receiver;

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
