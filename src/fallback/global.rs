use crate::fallback::{
    FallbackTryDropStrategy, GlobalFallbackTryDropStrategy as GlobalFallbackTryDropStrategyTrait,
};
use once_cell::sync::OnceCell;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::boxed::Box;

static FALLBACK_TRY_DROP_STRATEGY: OnceCell<RwLock<Box<dyn GlobalFallbackTryDropStrategyTrait>>> =
    OnceCell::new();

#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct GlobalFallbackTryDropStrategyHandler;

impl FallbackTryDropStrategy for GlobalFallbackTryDropStrategyHandler {
    fn handle_error_in_strategy(&self, error: anyhow::Error) {
        read().handle_error_in_strategy(error)
    }
}

fn fallback_drop_strategy() -> &'static RwLock<Box<dyn GlobalFallbackTryDropStrategyTrait>> {
    FALLBACK_TRY_DROP_STRATEGY.get()
        .expect("the global fallback try drop strategy is not initialized yet; initialize it with `global::initialize()`")
}

pub fn read() -> RwLockReadGuard<'static, Box<dyn GlobalFallbackTryDropStrategyTrait>> {
    fallback_drop_strategy().read()
}

pub fn write() -> RwLockWriteGuard<'static, Box<dyn GlobalFallbackTryDropStrategyTrait>> {
    fallback_drop_strategy().write()
}

pub fn install(drop_strategy: impl GlobalFallbackTryDropStrategyTrait) {
    install_dyn(Box::new(drop_strategy))
}

pub fn install_dyn(drop_strategy: Box<dyn GlobalFallbackTryDropStrategyTrait>) {
    match FALLBACK_TRY_DROP_STRATEGY.get() {
        Some(global_fallback_drop_strategy) => *global_fallback_drop_strategy.write() = drop_strategy,
        None => {
            let _ = FALLBACK_TRY_DROP_STRATEGY.set(RwLock::new(drop_strategy));
        }
    }
}
