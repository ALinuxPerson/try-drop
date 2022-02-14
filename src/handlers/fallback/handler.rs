use std::marker::PhantomData;
use anyhow::Error;
use crate::handlers::common::{Fallback, Global, Scope, ThreadLocal};
use crate::handlers::common::handler::CommonHandler;
use crate::handlers::common::proxy::TheGreatAbstracter;
use crate::handlers::on_uninit::{FlagOnUninit, PanicOnUninit, UseDefaultOnUninit};
use crate::handlers::UninitializedError;
use crate::TryDropStrategy;

/// The default thing to do when the global fallback handler is not initialized.
#[cfg(not(feature = "ds-panic"))]
pub type DefaultOnUninit = PanicOnUninit;

/// The default thing to do when the global fallback handler is not initialized.
#[cfg(feature = "ds-panic")]
pub type DefaultOnUninit = UseDefaultOnUninit;

type Abstracter<S> = TheGreatAbstracter<Fallback, S>;
pub type GlobalFallbackHandler<OU = DefaultOnUninit> = CommonHandler<OU, Global, Fallback>;
pub type ThreadLocalFallbackHandler<OU = DefaultOnUninit> = CommonHandler<OU, ThreadLocal, Fallback>;

pub static DEFAULT_GLOBAL_FALLBACK_HANDLER: GlobalFallbackHandler = GlobalFallbackHandler::DEFAULT;
pub static DEFAULT_THREAD_LOCAL_FALLBACK_HANDLER: ThreadLocalFallbackHandler = ThreadLocalFallbackHandler::DEFAULT;

impl<S: Scope> CommonHandler<DefaultOnUninit, S, Fallback> {
    pub const DEFAULT: Self = Self {
        extra_data: (),
        _scope: PhantomData,
    };
}

impl<S: Scope> Default for CommonHandler<DefaultOnUninit, S, Fallback> {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl TryDropStrategy for CommonHandler<PanicOnUninit, ThreadLocal, Fallback> {
    fn handle_error(&self, error: crate::Error) {
        Abstracter::<ThreadLocal>::read(|strategy| strategy.handle_error(error))
    }
}

impl TryDropStrategy for CommonHandler<PanicOnUninit, Global, Fallback> {
    fn handle_error(&self, error: crate::Error) {
        Abstracter::<Global>::read(|strategy| strategy.handle_error(error))
    }
}

#[cfg(feature = "ds-write")]
impl TryDropStrategy for CommonHandler<UseDefaultOnUninit, ThreadLocal, Fallback> {
    fn handle_error(&self, error: Error) {
        Abstracter::<ThreadLocal>::read_or_default(|strategy| strategy.handle_error(error))
    }
}

#[cfg(feature = "ds-write")]
impl TryDropStrategy for CommonHandler<UseDefaultOnUninit, Global, Fallback> {
    fn handle_error(&self, error: Error) {
        Abstracter::<Global>::read_or_default(|strategy| strategy.handle_error(error))
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

impl TryDropStrategy for CommonHandler<FlagOnUninit, Global, Fallback> {
    fn handle_error(&self, error: Error) {
        if let Err(UninitializedError(())) = Abstracter::<Global>::try_read(|strategy| strategy.handle_error(error)) {
            self.set_last_drop_failed(true)
        } else {
            self.set_last_drop_failed(false)
        }
    }
}

