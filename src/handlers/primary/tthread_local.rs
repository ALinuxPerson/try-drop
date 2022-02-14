use std::boxed::Box;
use std::cell::RefCell;
use std::thread::LocalKey;
use std::thread_local;
use crate::{DynFallibleTryDropStrategy, ThreadLocalFallibleTryDropStrategy};
use crate::handlers::common::Primary;
use crate::handlers::common::thread_local::{DefaultThreadLocalDefinition, ThreadLocal as GenericThreadLocal, scope_guard::ScopeGuard as GenericScopeGuard, ThreadLocalDefinition};

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

thread_local_methods! {
    ThreadLocal = ThreadLocal;
    ScopeGuard = ScopeGuard;
    GenericStrategy = ThreadLocalFallibleTryDropStrategy;
    DynStrategy = DynFallibleTryDropStrategy;
    feature = "ds-panic";
    install;
    install_dyn;
    try_read;
    read_or_default;
    write;
    try_write;
    write_or_default;
    uninstall;
    take;
    replace;
    replace_dyn;
    scope;
    scope_dyn;
}
