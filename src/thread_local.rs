//! Manage the thread local drop strategy.

#[cfg(feature = "ds-write")]
mod drop_strategy {
    use std::boxed::Box;
    use std::cell::RefCell;
    use std::thread_local;
    use once_cell::unsync::Lazy;
    use crate::DynFallibleTryDropStrategy;
    use crate::drop_strategies::WriteDropStrategy;

    thread_local! {
        static DROP_STRATEGY: Lazy<RefCell<Box<dyn DynFallibleTryDropStrategy>>> = Lazy::new(|| {
            let mut strategy = WriteDropStrategy::stderr();
            strategy.prelude("error: ");
            RefCell::new(Box::new(strategy))
        })
    }

    pub fn drop_strategy<T>(f: impl FnOnce(&RefCell<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> T {
        DROP_STRATEGY.with(|drop_strategy| f(drop_strategy))
    }

    /// Install this drop strategy to the current thread.
    pub fn install_dyn(strategy: Box<dyn DynFallibleTryDropStrategy>) {
        drop_strategy(|drop_strategy| *drop_strategy.borrow_mut() = strategy)
    }
}

#[cfg(not(feature = "ds-write"))]
mod drop_strategy {
    use std::thread_local;
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;
    use crate::DynFallibleTryDropStrategy;
    use std::boxed::Box;

    thread_local! {
        static DROP_STRATEGY: OnceCell<RefCell<Box<dyn DynFallibleTryDropStrategy>>> = OnceCell::new();
    }

    pub fn drop_strategy<T>(f: impl FnOnce(&RefCell<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> T {
        DROP_STRATEGY.with(|drop_strategy| {
            f(drop_strategy.get().expect("the thread local drop strategy is not initialized yet"))
        })
    }

    /// Install this drop strategy to the current thread.
    pub fn install_dyn(strategy: Box<dyn DynFallibleTryDropStrategy>) {
        DROP_STRATEGY.with(|drop_strategy| {
            match drop_strategy.get() {
                Some(thread_local_strategy) => *thread_local_strategy.borrow_mut() = strategy,
                None => {
                    let _ = drop_strategy.set(RefCell::new(strategy));
                }
            }
        })
    }
}

pub use drop_strategy::install_dyn;
use drop_strategy::drop_strategy;
use std::boxed::Box;
use std::cell::{Ref, RefCell, RefMut};
use anyhow::Error;
use once_cell::unsync::{Lazy, OnceCell};
use crate::{DynFallibleTryDropStrategy, FallibleTryDropStrategy};
use crate::utils::NotSendNotSync;

/// The thread local try drop strategy. This doesn't store anything, it just provides an interface
/// to the thread local try drop strategy, stored in a `static`.
///
/// # Note
/// This does **NOT** implement Send nor Sync because it not guaranteed that another thread will
/// have the same drop strategies as the thread that created this object; it could potentially be a
/// logic error. You can just create it on another thread as creating this is zero cost.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
struct ThreadLocalDropStrategy(NotSendNotSync);

impl ThreadLocalDropStrategy {
    /// Get the thread local try drop strategy.
    pub const fn new() -> Self {
        Self(NotSendNotSync::new())
    }
}

impl FallibleTryDropStrategy for ThreadLocalDropStrategy {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        read(|strategy| strategy.dyn_try_handle_error(error))
    }
}

/// Install a new thread local try drop strategy. Since this drop strategy will only be used in one
/// thread, it is more flexible than the global try drop strategy.
pub fn install(strategy: impl DynFallibleTryDropStrategy + 'static) {
    install_dyn(Box::new(strategy))
}

/// Get a reference to the thread local try drop strategy.
pub fn read<T>(f: impl FnOnce(Ref<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> T {
    drop_strategy(|strategy| f(strategy.borrow()))
}

/// Get a mutable reference to the thread local try drop strategy.
pub fn write<T>(f: impl FnOnce(RefMut<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> T {
    drop_strategy(|strategy| f(strategy.borrow_mut()))
}
