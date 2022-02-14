//! Manage the global fallback handler.

use crate::handlers::on_uninit::{FlagOnUninit, OnUninit, PanicOnUninit};
use crate::handlers::uninit_error::UninitializedError;
use crate::{GlobalTryDropStrategy, TryDropStrategy, LOAD_ORDERING, STORE_ORDERING};
use anyhow::Error;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock,
};
use std::boxed::Box;
use crate::handlers::common::Fallback;
use crate::handlers::common::global::{DefaultGlobalDefinition, GlobalDefinition, Global as GenericGlobal};
use crate::handlers::common::handler::CommonHandler;
use crate::handlers::common::Global as GlobalHandler;
use crate::handlers::fallback::Abstracter;
use super::DefaultOnUninit;

#[cfg(feature = "ds-panic")]
use crate::handlers::on_uninit::UseDefaultOnUninit;

pub type GlobalFallbackHandler<OU = DefaultOnUninit> = CommonHandler<OU, GlobalHandler, Fallback>;
pub static DEFAULT_GLOBAL_FALLBACK_HANDLER: GlobalFallbackHandler = GlobalFallbackHandler::DEFAULT;
static FALLBACK_HANDLER: RwLock<Option<Box<dyn GlobalTryDropStrategy>>> = parking_lot::const_rwlock(None);

impl TryDropStrategy for GlobalFallbackHandler<PanicOnUninit> {
    fn handle_error(&self, error: crate::Error) {
        Abstracter::<GlobalHandler>::read(|strategy| strategy.handle_error(error))
    }
}

#[cfg(feature = "ds-write")]
impl TryDropStrategy for GlobalFallbackHandler<UseDefaultOnUninit> {
    fn handle_error(&self, error: Error) {
        Abstracter::<GlobalHandler>::read_or_default(|strategy| strategy.handle_error(error))
    }
}

impl TryDropStrategy for GlobalFallbackHandler<FlagOnUninit> {
    fn handle_error(&self, error: Error) {
        if let Err(UninitializedError(())) = Abstracter::<GlobalHandler>::try_read(|strategy| strategy.handle_error(error)) {
            self.set_last_drop_failed(true)
        } else {
            self.set_last_drop_failed(false)
        }
    }
}

impl GlobalDefinition for Fallback {
    const UNINITIALIZED_ERROR: &'static str = "the global fallback handler is not initialized yet";
    type Global = Box<dyn GlobalTryDropStrategy>;

    fn global() -> &'static RwLock<Option<Self::Global>> {
        &FALLBACK_HANDLER
    }
}

#[cfg(feature = "ds-panic")]
impl DefaultGlobalDefinition for Fallback {
    fn default() -> Self::Global {
        Box::new(crate::drop_strategies::PanicDropStrategy::DEFAULT)
    }
}

impl<T: GlobalTryDropStrategy> From<T> for Box<dyn GlobalTryDropStrategy> {
    fn from(t: T) -> Self {
        Box::new(t)
    }
}

type Global = GenericGlobal<Fallback>;
type BoxDynGlobalTryDropStrategy = Box<dyn GlobalTryDropStrategy>;

global_methods! {
    Global = Global;
    GenericStrategy = GlobalTryDropStrategy;
    DynStrategy = BoxDynGlobalTryDropStrategy;
    feature = "ds-panic";

    install_dyn;
    install;
    try_read;
    read;
    try_write;
    write;
    uninstall;
    read_or_default;
    write_or_default;
}
