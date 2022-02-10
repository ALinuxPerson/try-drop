use std::boxed::Box;
use std::cell::{Ref, RefCell, RefMut};
use std::thread_local;
use anyhow::Error;
use once_cell::unsync::{Lazy, OnceCell};
use crate::{DynFallibleTryDropStrategy, FallibleTryDropStrategy};

thread_local! {
    static DROP_STRATEGY: OnceCell<RefCell<Box<dyn DynFallibleTryDropStrategy>>> = OnceCell::new();
}

struct ThreadLocalDropStrategy;

impl FallibleTryDropStrategy for ThreadLocalDropStrategy {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        read(|strategy| strategy.dyn_try_handle_error(error))
    }
}

fn drop_strategy<T>(f: impl FnOnce(&RefCell<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> T {
    DROP_STRATEGY.with(|drop_strategy| {
        f(drop_strategy.get().expect("the thread local drop strategy is not initialized yet"))
    })
}

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

pub fn install(strategy: impl DynFallibleTryDropStrategy + 'static) {
    install_dyn(Box::new(strategy))
}

pub fn read<T>(f: impl FnOnce(Ref<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> T {
    drop_strategy(|strategy| f(strategy.borrow()))
}

pub fn write<T>(f: impl FnOnce(RefMut<Box<dyn DynFallibleTryDropStrategy>>) -> T) -> T {
    drop_strategy(|strategy| f(strategy.borrow_mut()))
}
