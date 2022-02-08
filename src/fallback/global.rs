//! Manage the global fallback try drop strategy.
#[cfg(all(feature = "ds-panic", feature = "std"))]
mod fallback_drop_strategy {
    use std::boxed::Box;
    use once_cell::sync::Lazy;
    use parking_lot::RwLock;
    use crate::drop_strategies::PanicDropStrategy;
    use crate::fallback::GlobalFallbackTryDropStrategy as GlobalFallbackTryDropStrategyTrait;

    static FALLBACK_TRY_DROP_STRATEGY: Lazy<RwLock<Box<dyn GlobalFallbackTryDropStrategyTrait>>> =
        Lazy::new(|| RwLock::new(Box::new(PanicDropStrategy::DEFAULT)));

    pub fn fallback_drop_strategy() -> &'static RwLock<Box<dyn GlobalFallbackTryDropStrategyTrait>> {
        &FALLBACK_TRY_DROP_STRATEGY
    }

    /// Install a new global fallback try drop strategy. Needs to be a dynamic trait object.
    pub fn install_dyn(drop_strategy: Box<dyn GlobalFallbackTryDropStrategyTrait>) {
        *fallback_drop_strategy().write() = drop_strategy
    }
}
#[cfg(not(all(feature = "ds-panic", feature = "std")))]
mod fallback_drop_strategy {
    use std::boxed::Box;
    use once_cell::sync::OnceCell;
    use parking_lot::RwLock;
    use crate::fallback::GlobalFallbackTryDropStrategy as GlobalFallbackTryDropStrategyTrait;

    static FALLBACK_TRY_DROP_STRATEGY: OnceCell<RwLock<Box<dyn GlobalFallbackTryDropStrategyTrait>>> =
        OnceCell::new();

    pub fn fallback_drop_strategy() -> &'static RwLock<Box<dyn GlobalFallbackTryDropStrategyTrait>> {
        FALLBACK_TRY_DROP_STRATEGY.get()
            .expect("the global fallback try drop strategy is not initialized yet; initialize it with `global::initialize()`")
    }

    /// Install a new global fallback try drop strategy. Needs to be a dynamic trait object.
    pub fn install_dyn(drop_strategy: Box<dyn GlobalFallbackTryDropStrategyTrait>) {
        match FALLBACK_TRY_DROP_STRATEGY.get() {
            Some(global_fallback_drop_strategy) => {
                *global_fallback_drop_strategy.write() = drop_strategy
            }
            None => {
                let _ = FALLBACK_TRY_DROP_STRATEGY.set(RwLock::new(drop_strategy));
            }
        }
    }
}

pub use fallback_drop_strategy::install_dyn;
use fallback_drop_strategy::fallback_drop_strategy;
use crate::fallback::{
    FallbackTryDropStrategy, GlobalFallbackTryDropStrategy as GlobalFallbackTryDropStrategyTrait,
};
use once_cell::sync::OnceCell;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::boxed::Box;

/// The global fallback try drop strategy. This doesn't store anything, it just provides a
/// interface to the global fallback try drop strategy, stored in a `static`.
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

/// Get a reference to the global fallback try drop strategy.
pub fn read() -> RwLockReadGuard<'static, Box<dyn GlobalFallbackTryDropStrategyTrait>> {
    fallback_drop_strategy().read()
}

/// Get a mutable reference to the global fallback try drop strategy.
pub fn write() -> RwLockWriteGuard<'static, Box<dyn GlobalFallbackTryDropStrategyTrait>> {
    fallback_drop_strategy().write()
}

/// Install a new global fallback try drop strategy.
pub fn install(drop_strategy: impl GlobalFallbackTryDropStrategyTrait) {
    install_dyn(Box::new(drop_strategy))
}
