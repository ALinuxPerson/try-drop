use std::boxed::Box;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock};
use crate::handlers::common::Fallback;
use crate::handlers::common::global::{DefaultGlobalDefinition, Global as GenericGlobal, GlobalDefinition};
use crate::GlobalTryDropStrategy;
use crate::drop_strategies::PanicDropStrategy;
use crate::handlers::UninitializedError;

static FALLBACK_HANDLER: RwLock<Option<Box<dyn GlobalTryDropStrategy>>> = parking_lot::const_rwlock(None);

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
        Box::new(PanicDropStrategy::DEFAULT)
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
