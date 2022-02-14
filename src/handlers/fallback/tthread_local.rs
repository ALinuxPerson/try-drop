use std::boxed::Box;
use std::cell::RefCell;
use std::thread::LocalKey;
use std::thread_local;
use crate::handlers::common::Fallback;
use crate::handlers::common::thread_local::scope_guard::ScopeGuard as GenericScopeGuard;
use crate::handlers::common::thread_local::{DefaultThreadLocalDefinition, ThreadLocal as GenericThreadLocal, ThreadLocalDefinition};
use crate::{ThreadLocalTryDropStrategy, TryDropStrategy};

thread_local! {
    static FALLBACK_HANDLER: RefCell<Option<Box<dyn ThreadLocalTryDropStrategy>>> = RefCell::new(None);
    static LOCKED: RefCell<bool> = RefCell::new(false);
}

impl ThreadLocalDefinition for Fallback {
    const UNINITIALIZED_ERROR: &'static str = "the thread local fallback handler is not initialized yet";
    const DYN: &'static str = "TryDropStrategy";
    type ThreadLocal = Box<dyn ThreadLocalTryDropStrategy>;

    fn thread_local() -> &'static LocalKey<RefCell<Option<Self::ThreadLocal>>> {
        &FALLBACK_HANDLER
    }

    fn locked() -> &'static LocalKey<RefCell<bool>> {
        &LOCKED
    }
}

#[cfg(feature = "ds-panic")]
impl DefaultThreadLocalDefinition for Fallback {
    fn default() -> Self::ThreadLocal {
        Box::new(crate::drop_strategies::PanicDropStrategy::DEFAULT)
    }
}

impl<T: ThreadLocalTryDropStrategy> From<T> for Box<dyn ThreadLocalTryDropStrategy> {
    fn from(strategy: T) -> Self {
        Box::new(strategy)
    }
}

type ThreadLocal = GenericThreadLocal<Fallback>;
pub type ScopeGuard = GenericScopeGuard<Fallback>;
pub type BoxDynTryDropStrategy = Box<dyn ThreadLocalTryDropStrategy>;

thread_local_methods! {
    ThreadLocal = ThreadLocal;
    ScopeGuard = ScopeGuard;
    GenericStrategy = ThreadLocalTryDropStrategy;
    DynStrategy = BoxDynTryDropStrategy;
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

