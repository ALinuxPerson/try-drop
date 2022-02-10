//! Manage the global try drop strategy.

#[cfg(feature = "ds-write")]
mod drop_strategy {
    use crate::drop_strategies::WriteDropStrategy;
    use crate::GlobalDynFallibleTryDropStrategy;
    use once_cell::sync::Lazy;
    use parking_lot::RwLock;
    use std::boxed::Box;

    static DROP_STRATEGY: Lazy<RwLock<Box<dyn GlobalDynFallibleTryDropStrategy>>> =
        Lazy::new(|| {
            let mut strategy = WriteDropStrategy::stderr();
            strategy.prelude("error: ");
            RwLock::new(Box::new(strategy))
        });

    pub fn drop_strategy() -> &'static RwLock<Box<dyn GlobalDynFallibleTryDropStrategy>> {
        &DROP_STRATEGY
    }

    /// Install a new global try drop strategy. Needs to be a dynamic trait object.
    pub fn install_dyn(strategy: Box<dyn GlobalDynFallibleTryDropStrategy>) {
        *drop_strategy().write() = strategy
    }

    /// Check if there is already a drop strategy in this global.
    pub fn initialized() -> bool {
        true
    }
}

#[cfg(not(feature = "ds-write"))]
mod drop_strategy {
    use crate::GlobalDynFallibleTryDropStrategy;
    use once_cell::sync::OnceCell;
    use parking_lot::RwLock;
    use std::boxed::Box;

    static DROP_STRATEGY: OnceCell<RwLock<Box<dyn GlobalDynFallibleTryDropStrategy>>> =
        OnceCell::new();

    pub fn drop_strategy() -> &'static RwLock<Box<dyn GlobalDynFallibleTryDropStrategy>> {
        DROP_STRATEGY.get().expect(
            "the global drop strategy is not initialized; initialize it with `global::install()`",
        )
    }

    /// Install a new global try drop strategy. Needs to be a dynamic trait object.
    pub fn install_dyn(drop_strategy: Box<dyn GlobalDynFallibleTryDropStrategy>) {
        match DROP_STRATEGY.get() {
            Some(global_drop_strategy) => *global_drop_strategy.write() = drop_strategy,
            None => {
                let _ = DROP_STRATEGY.set(RwLock::new(drop_strategy));
            }
        }
    }

    /// Check if there is already a drop strategy in this global.
    pub fn initialized() -> bool {
        DROP_STRATEGY.get().is_some()
    }
}

use crate::{FallibleTryDropStrategy, GlobalDynFallibleTryDropStrategy};
use anyhow::Error;
use drop_strategy::drop_strategy;
pub use drop_strategy::{install_dyn, initialized};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use std::boxed::Box;

/// The global try drop strategy. This doesn't store anything, it just provides an interface
/// to the global try drop strategy, stored in a `static`.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct GlobalTryDropStrategyHandler;

impl FallibleTryDropStrategy for GlobalTryDropStrategyHandler {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        read().dyn_try_handle_error(error)
    }
}

/// Get a reference to the global try drop strategy.
pub fn read() -> RwLockReadGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    drop_strategy().read()
}

/// Get a mutable reference to the global try drop strategy.
pub fn write() -> RwLockWriteGuard<'static, Box<dyn GlobalDynFallibleTryDropStrategy>> {
    drop_strategy().write()
}

/// Install a new global try drop strategy. Since this drop strategy will only be used in one
/// thread, it is more flexible than the global try drop strategy.
pub fn install(drop_strategy: impl GlobalDynFallibleTryDropStrategy) {
    install_dyn(Box::new(drop_strategy))
}
