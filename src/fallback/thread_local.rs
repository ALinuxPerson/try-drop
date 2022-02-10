#[cfg(feature = "ds-panic")]
mod drop_strategy {
    use std::cell::RefCell;
    use std::thread_local;
    use once_cell::unsync::Lazy;
    use crate::drop_strategies::PanicDropStrategy;
    use crate::FallbackTryDropStrategy;
    use std::boxed::Box;

    thread_local! {
        static FALLBACK_TRY_DROP_STRATEGY: Lazy<RefCell<Box<dyn FallbackTryDropStrategy>>> = Lazy::new(|| {
            RefCell::new(Box::new(PanicDropStrategy::DEFAULT))
        });
    }

    pub fn fallback_try_drop_strategy<T>(f: impl FnOnce(&RefCell<Box<dyn FallbackTryDropStrategy>>) -> T) -> T {
        FALLBACK_TRY_DROP_STRATEGY.with(|fallback_try_drop_strategy| {
            f(fallback_try_drop_strategy)
        })
    }

    pub fn install_dyn(strategy: Box<dyn FallbackTryDropStrategy>) {
        FALLBACK_TRY_DROP_STRATEGY.with(|fallback_try_drop_strategy| {
            *fallback_try_drop_strategy.borrow_mut() = strategy;
        });
    }

    pub fn initialized() -> bool {
        true
    }
}

#[cfg(not(feature = "ds-panic"))]
mod drop_strategy {
    use std::boxed::Box;
    use std::cell::RefCell;
    use std::thread_local;
    use once_cell::unsync::OnceCell;
    use crate::FallbackTryDropStrategy;

    thread_local! {
        static FALLBACK_TRY_DROP_STRATEGY: OnceCell<RefCell<Box<dyn FallbackTryDropStrategy>>> = OnceCell::new();
    }

    pub fn fallback_try_drop_strategy<T>(f: impl FnOnce(&RefCell<Box<dyn FallbackTryDropStrategy>>) -> T) -> T {
        FALLBACK_TRY_DROP_STRATEGY.with(|drop_strategy| {
            f(drop_strategy.get().expect("the thread local fallback drop strategy is not initialized yet"))
        })
    }

    pub fn install_dyn(strategy: Box<dyn FallbackTryDropStrategy>) {
        FALLBACK_TRY_DROP_STRATEGY.with(|drop_strategy| {
            match drop_strategy.get() {
                Some(thread_local_strategy) => *thread_local_strategy.borrow_mut() = strategy,
                None => {
                    let _ = drop_strategy.set(RefCell::new(strategy));
                }
            }
        })
    }

    pub fn initialized() -> bool {
        FALLBACK_TRY_DROP_STRATEGY.with(|drop_strategy| {
            drop_strategy.get().is_some()
        })
    }
}

use std::boxed::Box;
use std::cell::{Ref, RefMut};
pub use drop_strategy::{initialized, install_dyn};
use drop_strategy::fallback_try_drop_strategy;
use crate::{FallbackTryDropStrategy, TryDropStrategy};

pub struct ThreadLocalFallbackTryDropStrategy;

impl TryDropStrategy for ThreadLocalFallbackTryDropStrategy {
    fn handle_error(&self, error: crate::Error) {
        read(|strategy| strategy.handle_error_in_strategy(error))
    }
}

pub fn read<T>(f: impl FnOnce(Ref<Box<dyn FallbackTryDropStrategy>>) -> T) -> T {
    fallback_try_drop_strategy(|strategy| f(strategy.borrow()))
}

pub fn write<T>(f: impl FnOnce(RefMut<Box<dyn FallbackTryDropStrategy>>) -> T) -> T {
    fallback_try_drop_strategy(|strategy| f(strategy.borrow_mut()))
}

pub fn install(strategy: impl FallbackTryDropStrategy + 'static) {
    install_dyn(Box::new(strategy))
}