//! Manage the global fallback handler.

use super::DefaultOnUninit;
use crate::handlers::common::global::{
    DefaultGlobalDefinition, Global as GenericGlobal, GlobalDefinition,
};
use crate::handlers::common::handler::CommonHandler;
use crate::handlers::common::Fallback;
use crate::handlers::common::Global as GlobalScope;
use crate::handlers::fallback::Abstracter;
use crate::handlers::on_uninit::{FlagOnUninit, OnUninit, PanicOnUninit};
use crate::handlers::uninit_error::UninitializedError;
use crate::{GlobalTryDropStrategy, TryDropStrategy, LOAD_ORDERING, STORE_ORDERING};
use anyhow::Error;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock};
use std::boxed::Box;

#[cfg(feature = "ds-panic")]
use crate::handlers::on_uninit::UseDefaultOnUninit;

pub type GlobalFallbackHandler<OU = DefaultOnUninit> = CommonHandler<OU, GlobalScope, Fallback>;
pub static DEFAULT_GLOBAL_FALLBACK_HANDLER: GlobalFallbackHandler = GlobalFallbackHandler::DEFAULT;
static FALLBACK_HANDLER: RwLock<Option<Box<dyn GlobalTryDropStrategy>>> =
    parking_lot::const_rwlock(None);

impl_try_drop_strategy_for!(GlobalFallbackHandler where Scope: GlobalScope);

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
