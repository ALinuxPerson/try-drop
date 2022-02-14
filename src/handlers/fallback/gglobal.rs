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

pub fn install_dyn(strategy: Box<dyn GlobalTryDropStrategy>) {
    Global::install_dyn(strategy)
}

pub fn install(strategy: impl GlobalTryDropStrategy) {
    Global::install(strategy)
}

pub fn try_read() -> Result<MappedRwLockReadGuard<'static, Box<dyn GlobalTryDropStrategy>>, UninitializedError> {
    Global::try_read()
}

pub fn read() -> MappedRwLockReadGuard<'static, Box<dyn GlobalTryDropStrategy>> {
    Global::read()
}

pub fn try_write() -> Result<MappedRwLockWriteGuard<'static, Box<dyn GlobalTryDropStrategy>>, UninitializedError> {
    Global::try_write()
}

pub fn write() -> MappedRwLockWriteGuard<'static, Box<dyn GlobalTryDropStrategy>> {
    Global::write()
}

pub fn uninstall() {
    Global::uninstall()
}

#[cfg(feature = "ds-panic")]
pub fn read_or_default() -> MappedRwLockReadGuard<'static, Box<dyn GlobalTryDropStrategy>> {
    Global::read_or_default()
}

#[cfg(feature = "ds-panic")]
pub fn write_or_default() -> MappedRwLockWriteGuard<'static, Box<dyn GlobalTryDropStrategy>> {
    Global::write_or_default()
}
