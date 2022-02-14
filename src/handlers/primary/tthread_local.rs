use std::boxed::Box;
use std::cell::RefCell;
use std::thread::LocalKey;
use std::thread_local;
use crate::{DynFallibleTryDropStrategy, ThreadLocalFallibleTryDropStrategy};
use crate::handlers::common::Primary;
use crate::handlers::common::thread_local::{
    DefaultThreadLocalDefinition,
    ThreadLocal as GenericThreadLocal,
    scope_guard::ScopeGuard as GenericScopeGuard,
    ThreadLocalDefinition,
};
use crate::handlers::UninitializedError;

thread_local! {
    static PRIMARY_HANDLER: RefCell<Option<Box<dyn DynFallibleTryDropStrategy>>> = RefCell::new(None);
    static LOCKED: RefCell<bool> = RefCell::new(false);
}

impl ThreadLocalDefinition for Primary {
    const UNINITIALIZED_ERROR: &'static str = "the thread local primary handler is not initialized yet";
    const DYN: &'static str = "DynFallibleTryDropStrategy";
    type ThreadLocal = Box<dyn DynFallibleTryDropStrategy>;

    fn thread_local() -> &'static LocalKey<RefCell<Option<Self::ThreadLocal>>> {
        &PRIMARY_HANDLER
    }

    fn locked() -> &'static LocalKey<RefCell<bool>> {
        &LOCKED
    }
}

#[cfg(feature = "ds-write")]
impl DefaultThreadLocalDefinition for Primary {
    fn default() -> Self::ThreadLocal {
        let mut strategy = crate::drop_strategies::WriteDropStrategy::stderr();
        strategy.prelude("error: ");
        Box::new(strategy)
    }
}

impl<T: ThreadLocalFallibleTryDropStrategy> From<T> for Box<dyn DynFallibleTryDropStrategy> {
    fn from(strategy: T) -> Self {
        Box::new(strategy)
    }
}

type ThreadLocal = GenericThreadLocal<Primary>;
pub type ScopeGuard = GenericScopeGuard<Primary>;

pub fn install(strategy: impl ThreadLocalFallibleTryDropStrategy) {
    ThreadLocal::install(strategy)
}

pub fn install_dyn(strategy: Box<dyn DynFallibleTryDropStrategy>) {
    ThreadLocal::install_dyn(strategy)
}

pub fn try_read<T>(f: impl FnOnce(&Box<dyn DynFallibleTryDropStrategy>) -> T) -> Result<T, UninitializedError> {
    ThreadLocal::try_read(f)
}

pub fn write<T>(f: impl FnOnce(&mut Box<dyn DynFallibleTryDropStrategy>) -> T) -> T {
    ThreadLocal::write(f)
}

pub fn try_write<T>(f: impl FnOnce(&mut Box<dyn DynFallibleTryDropStrategy>) -> T) -> Result<T, UninitializedError> {
    ThreadLocal::try_write(f)
}

pub fn uninstall() {
    ThreadLocal::uninstall()
}

pub fn take() -> Option<Box<dyn DynFallibleTryDropStrategy>> {
    ThreadLocal::take()
}

pub fn replace(strategy: impl ThreadLocalFallibleTryDropStrategy) -> Option<Box<dyn DynFallibleTryDropStrategy>> {
    ThreadLocal::replace(strategy)
}

pub fn replace_dyn(strategy: Box<dyn DynFallibleTryDropStrategy>) -> Option<Box<dyn DynFallibleTryDropStrategy>> {
    ThreadLocal::replace_dyn(strategy)
}

pub fn scope(strategy: impl ThreadLocalFallibleTryDropStrategy) -> ScopeGuard {
    ThreadLocal::scope(strategy)
}

pub fn scope_dyn(strategy: Box<dyn DynFallibleTryDropStrategy>) -> ScopeGuard {
    ThreadLocal::scope_dyn(strategy)
}
