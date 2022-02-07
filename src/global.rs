use crate::{FallibleTryDropStrategy, GlobalDynFallibleTryDropStrategy};
use anyhow::Error;
use once_cell::sync::OnceCell;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::boxed::Box;

static DROP_STRATEGY: OnceCell<RwLock<Box<dyn GlobalDynFallibleTryDropStrategy>>> = OnceCell::new();

#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
pub struct GlobalDropStrategyHandler;

impl FallibleTryDropStrategy for GlobalDropStrategyHandler {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        read().dyn_try_handle_error(error)
    }
}

fn drop_strategy() -> &'static RwLock<Box<dyn GlobalDynFallibleTryDropStrategy>> {
    DROP_STRATEGY.get().expect(
        "the global drop strategy is not initialized; initialize it with `global::install()`",
    )
}

pub fn read() -> RwLockReadGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    drop_strategy().read()
}

pub fn write() -> RwLockWriteGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    drop_strategy().write()
}

pub fn install(drop_strategy: impl GlobalDynFallibleTryDropStrategy) {
    install_dyn(Box::new(drop_strategy))
}

pub fn install_dyn(drop_strategy: Box<dyn GlobalDynFallibleTryDropStrategy>) {
    match DROP_STRATEGY.get() {
        Some(global_drop_strategy) => *global_drop_strategy.write() = drop_strategy,
        None => {
            let _ = DROP_STRATEGY.set(RwLock::new(drop_strategy));
        }
    }
}
