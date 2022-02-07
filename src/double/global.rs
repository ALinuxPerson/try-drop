use crate::double::{
    DoubleDropStrategy, GlobalDoubleDropStrategy as GlobalDoubleDropStrategyTrait,
};
use once_cell::sync::OnceCell;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::boxed::Box;

static DOUBLE_DROP_STRATEGY: OnceCell<RwLock<Box<dyn GlobalDoubleDropStrategyTrait>>> =
    OnceCell::new();

#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct GlobalDoubleDropStrategyHandler;

impl DoubleDropStrategy for GlobalDoubleDropStrategyHandler {
    fn handle_error_in_drop_strategy(&self, error: anyhow::Error) {
        read().handle_error_in_drop_strategy(error)
    }
}

fn double_drop_strategy() -> &'static RwLock<Box<dyn GlobalDoubleDropStrategyTrait>> {
    DOUBLE_DROP_STRATEGY.get()
        .expect("the global double drop strategy is not initialized yet; initialize it with `global::initialize()`")
}

pub fn read() -> RwLockReadGuard<'static, Box<dyn GlobalDoubleDropStrategyTrait>> {
    double_drop_strategy().read()
}

pub fn write() -> RwLockWriteGuard<'static, Box<dyn GlobalDoubleDropStrategyTrait>> {
    double_drop_strategy().write()
}

pub fn install(drop_strategy: impl GlobalDoubleDropStrategyTrait) {
    install_dyn(Box::new(drop_strategy))
}

pub fn install_dyn(drop_strategy: Box<dyn GlobalDoubleDropStrategyTrait>) {
    match DOUBLE_DROP_STRATEGY.get() {
        Some(global_double_drop_strategy) => *global_double_drop_strategy.write() = drop_strategy,
        None => {
            let _ = DOUBLE_DROP_STRATEGY.set(RwLock::new(drop_strategy));
        }
    }
}
