//! Manage the thread local fallback handler.
use crate::handlers::on_uninit::{ErrorOnUninit, FlagOnUninit, OnUninit, PanicOnUninit};
use crate::handlers::uninit_error::UninitializedError;
use crate::{TryDropStrategy, LOAD_ORDERING, STORE_ORDERING};
use anyhow::Error;
use std::boxed::Box;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::thread_local;
use std::thread::LocalKey;
use crate::handlers::common::Fallback;
use crate::handlers::common::thread_local::scope_guard::ScopeGuard as GenericScopeGuard;
use crate::handlers::common::thread_local::{DefaultThreadLocalDefinition, ThreadLocal as GenericThreadLocal, ThreadLocalDefinition};
use crate::ThreadLocalTryDropStrategy;

#[cfg(feature = "ds-panic")]
use crate::handlers::on_uninit::UseDefaultOnUninit;

/// The default thing to do when the fallback handler is uninitialized.
#[cfg(not(feature = "ds-panic"))]
pub type DefaultOnUninit = PanicOnUninit;

/// The default thing to do when the fallback handler is uninitialized.
#[cfg(feature = "ds-panic")]
pub type DefaultOnUninit = UseDefaultOnUninit;

/// The default thread local fallback handler.
pub static DEFAULT_THREAD_LOCAL_FALLBACK_HANDLER: ThreadLocalFallbackHandler =
    ThreadLocalFallbackHandler::DEFAULT;

const UNINITIALIZED_ERROR: &str = "the thread local fallback handler is not initialized yet";

/// The thread local fallback handler. This doesn't store anything, it just provides an
/// interface to the thread local fallback handler, stored in a `static`.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct ThreadLocalFallbackHandler<OU: OnUninit = DefaultOnUninit> {
    extra_data: OU::ExtraData,
    _on_uninit: PhantomData<OU>,
}

impl ThreadLocalFallbackHandler<DefaultOnUninit> {
    /// The default thread local fallback handler.
    pub const DEFAULT: Self = Self {
        extra_data: (),
        _on_uninit: PhantomData,
    };
}

impl ThreadLocalFallbackHandler<PanicOnUninit> {
    /// See [`Self::on_uninit_panic`].
    pub const PANIC_ON_UNINIT: Self = Self::on_uninit_panic();

    /// Create a new interface to the thread local fallback handler. If the thread local fallback
    /// handler is not initialized, this will panic.
    pub const fn on_uninit_panic() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

#[cfg(feature = "ds-panic")]
impl ThreadLocalFallbackHandler<UseDefaultOnUninit> {
    /// See [`Self::on_uninit_use_default`].
    pub const USE_DEFAULT_ON_UNINIT: Self = Self::on_uninit_use_default();

    /// Create a new interface to the thread local fallback handler. If the thread local fallback
    /// handler is not initialized, this will set it to the default fallback handler.
    pub const fn on_uninit_use_default() -> Self {
        Self {
            extra_data: (),
            _on_uninit: PhantomData,
        }
    }
}

impl ThreadLocalFallbackHandler<FlagOnUninit> {
    /// See [`Self::on_uninit_flag`].
    #[allow(clippy::declare_interior_mutable_const)]
    pub const FLAG_ON_UNINIT: Self = Self::on_uninit_flag();

    /// Create a new interface to the thread local fallback handler. If the thread local fallback
    /// handler is not initialized, a flag `last_drop_failed` will be set to true.
    pub const fn on_uninit_flag() -> Self {
        Self {
            extra_data: AtomicBool::new(false),
            _on_uninit: PhantomData,
        }
    }

    /// Check if the last drop failed due to the thread local fallback handler not being
    /// initialized.
    pub fn last_drop_failed(&self) -> bool {
        self.extra_data.load(LOAD_ORDERING)
    }

    fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, STORE_ORDERING)
    }
}

impl TryDropStrategy for ThreadLocalFallbackHandler<PanicOnUninit> {
    fn handle_error(&self, error: Error) {
        read(|strategy| strategy.handle_error(error))
    }
}

#[cfg(feature = "ds-panic")]
impl TryDropStrategy for ThreadLocalFallbackHandler<UseDefaultOnUninit> {
    fn handle_error(&self, error: Error) {
        read_or_default(|strategy| strategy.handle_error(error))
    }
}

impl TryDropStrategy for ThreadLocalFallbackHandler<FlagOnUninit> {
    fn handle_error(&self, error: Error) {
        if let Err(UninitializedError(())) = try_read(|strategy| strategy.handle_error(error)) {
            self.set_last_drop_failed(true)
        } else {
            self.set_last_drop_failed(false)
        }
    }
}

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
    read;
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

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use anyhow::anyhow;
    use crate::drop_strategies::{AdHocFallibleTryDropStrategy, IntoAdHocTryDropStrategy, NoOpDropStrategy};
    use crate::handlers::{fallback, primary};
    use crate::PureTryDrop;
    use crate::test_utils::{ErrorsOnDrop, Fallible};
    use super::*;

    #[test]
    #[should_panic]
    fn test_panic_on_uninit() {
        ThreadLocalFallbackHandler::on_uninit_panic().handle_error(anyhow!("test"))
    }

    #[test]
    fn test_flag_on_uninit() {
        let handler = ThreadLocalFallbackHandler::on_uninit_flag();
        assert!(!handler.last_drop_failed(), "last drop error handle failed but we haven't dropped anything yet");
        handler.handle_error(anyhow!("test"));
        assert!(handler.last_drop_failed(), "last drop error handle didn't fail but we have dropped something while not initialized");
        install(NoOpDropStrategy);
        handler.handle_error(anyhow!("test"));
        assert!(!handler.last_drop_failed(), "last drop error handle failed but we have initialized");
    }

    #[test]
    fn test_install() {
        let installed = Rc::new(RefCell::new(false));
        let i = Rc::clone(&installed);
        install((move |_| *i.borrow_mut() = true).into_adhoc_try_drop_strategy());
        fallback::thread_local::install(AdHocFallibleTryDropStrategy(|error| Err(error)));
        drop(ErrorsOnDrop::<Fallible, _>::not_given().adapt());
        assert!(*installed.borrow(), "install didn't install");
    }

    #[test]
    fn test_install_dyn() {
        let installed = Rc::new(RefCell::new(false));
        let i = Rc::clone(&installed);
        install_dyn(Box::new((move |_| *i.borrow_mut() = true).into_adhoc_try_drop_strategy()));
        fallback::thread_local::install(AdHocFallibleTryDropStrategy(|error| Err(error)));
        drop(ErrorsOnDrop::<Fallible, _>::not_given().adapt());
        assert!(*installed.borrow(), "install_dyn didn't install");
    }

    #[test]
    #[should_panic(expected = "the thread local fallback handler is not initialized yet: UninitializedError(())")]
    fn test_read_panics_on_uninit() {
        read(|_| panic!("did not panic on uninit"))
    }

    #[test]
    fn test_try_read_errors_on_uninit() {
        try_read(|_| panic!("did not error on uninit")).expect_err("did not error on uninit");
    }

    #[test]
    fn test_read_or_default() {
        let mut executed = false;
        read_or_default(|_| executed = true);
        assert!(executed, "read_or_default didn't execute");
    }

    #[test]
    #[should_panic(expected = "the thread local fallback handler is not initialized yet: UninitializedError(())")]
    fn test_write_panics_on_uninit() {
        write(|_| panic!("did not panic on uninit"))
    }

    #[test]
    fn test_try_write_errors_on_uninit() {
        try_write(|_| panic!("did not error on uninit")).expect_err("did not error on uninit");
    }

    #[test]
    fn test_write_or_default() {
        let mut executed = false;
        write_or_default(|_| executed = true);
        assert!(executed, "read_or_default didn't execute");
    }
    // todo: test uninstall, take, replace, replace_dyn, scope, scope_dyn
}