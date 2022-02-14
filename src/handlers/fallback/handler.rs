use std::marker::PhantomData;
use anyhow::Error;
use crate::handlers::common::{Fallback, Global, Scope, ThreadLocal};
use crate::handlers::common::handler::CommonHandler;
use crate::handlers::common::proxy::TheGreatAbstracter;
use crate::handlers::on_uninit::{FlagOnUninit, PanicOnUninit, UseDefaultOnUninit};
use crate::handlers::UninitializedError;
use crate::TryDropStrategy;

pub type ThreadLocalFallbackHandler<OU = DefaultOnUninit> = CommonHandler<OU, ThreadLocal, Fallback>;

pub static DEFAULT_THREAD_LOCAL_FALLBACK_HANDLER: ThreadLocalFallbackHandler = ThreadLocalFallbackHandler::DEFAULT;



impl TryDropStrategy for CommonHandler<PanicOnUninit, ThreadLocal, Fallback> {
    fn handle_error(&self, error: crate::Error) {
        Abstracter::<ThreadLocal>::read(|strategy| strategy.handle_error(error))
    }
}

#[cfg(feature = "ds-write")]
impl TryDropStrategy for CommonHandler<UseDefaultOnUninit, ThreadLocal, Fallback> {
    fn handle_error(&self, error: Error) {
        Abstracter::<ThreadLocal>::read_or_default(|strategy| strategy.handle_error(error))
    }
}

impl TryDropStrategy for CommonHandler<FlagOnUninit, ThreadLocal, Fallback> {
    fn handle_error(&self, error: Error) {
        if let Err(UninitializedError(())) = Abstracter::<ThreadLocal>::try_read(|strategy| strategy.handle_error(error)) {
            self.set_last_drop_failed(true)
        } else {
            self.set_last_drop_failed(false)
        }
    }
}
