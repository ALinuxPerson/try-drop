use std::boxed::Box;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock};
use crate::GlobalDynFallibleTryDropStrategy;
use crate::handlers::common::Fallback;
use crate::handlers::common::global::{DefaultGlobalDefinition, Global as GenericGlobal, GlobalDefinition};
use crate::handlers::UninitializedError;

static PRIMARY_HANDLER: RwLock<Option<Box<dyn GlobalDynFallibleTryDropStrategy>>> =
    parking_lot::const_rwlock(None);

impl GlobalDefinition for Fallback {
    const UNINITIALIZED_ERROR: &'static str = "the global primary handler is not initialized yet";
    type Global = Box<dyn GlobalDynFallibleTryDropStrategy>;

    fn global() -> &'static RwLock<Option<Self::Global>> {
        &PRIMARY_HANDLER
    }
}

#[cfg(feature = "ds-write")]
impl DefaultGlobalDefinition for Fallback {
    fn default() -> Self::Global {
        let mut strategy = crate::drop_strategies::WriteDropStrategy::stderr();
        strategy.prelude("error: ");
        Box::new(strategy)
    }
}

impl<T: GlobalDynFallibleTryDropStrategy + 'static> From<T> for Box<dyn GlobalDynFallibleTryDropStrategy> {
    fn from(handler: T) -> Self {
        Box::new(handler)
    }
}

type Global = GenericGlobal<Fallback>;

pub fn install_dyn(strategy: Box<dyn GlobalDynFallibleTryDropStrategy>) {
    Global::install_dyn(strategy)
}

pub fn install(strategy: impl GlobalDynFallibleTryDropStrategy + 'static) {
    Global::install(strategy)
}

pub fn try_read() -> Result<MappedRwLockReadGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>>, UninitializedError> {
    Global::try_read()
}

pub fn read() -> MappedRwLockReadGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    Global::read()
}

pub fn try_write() -> Result<MappedRwLockWriteGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>>, UninitializedError> {
    Global::try_write()
}

pub fn write() -> MappedRwLockWriteGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    Global::write()
}

pub fn uninstall() {
    Global::uninstall()
}

#[cfg(feature = "ds-write")]
pub fn read_or_default() -> MappedRwLockReadGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    Global::read_or_default()
}

#[cfg(feature = "ds-write")]
pub fn write_or_default() -> MappedRwLockWriteGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    Global::write_or_default()
}


