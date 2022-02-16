//! Manage the thread local fallback handler.
use super::{Abstracter, DefaultOnUninit};
use crate::handlers::common::handler::CommonHandler;
use crate::handlers::common::thread_local::scope_guard::ScopeGuard as GenericScopeGuard;
use crate::handlers::common::thread_local::{
    ThreadLocal as GenericThreadLocal, ThreadLocalDefinition,
};
use crate::handlers::common::{Fallback, ThreadLocal as ThreadLocalScope};
use crate::handlers::on_uninit::{FlagOnUninit, PanicOnUninit};
use crate::handlers::uninit_error::UninitializedError;
use crate::ThreadLocalTryDropStrategy;
use crate::TryDropStrategy;
use anyhow::Error;
use std::boxed::Box;
use std::cell::RefCell;
use std::thread::LocalKey;
use std::thread_local;

#[cfg(feature = "ds-panic")]
use crate::handlers::common::thread_local::DefaultThreadLocalDefinition;

#[cfg(feature = "ds-panic")]
use crate::handlers::on_uninit::UseDefaultOnUninit;

/// A fallback handler that uses the thread local scope.
pub type ThreadLocalFallbackHandler<OU = DefaultOnUninit> =
    CommonHandler<OU, ThreadLocalScope, Fallback>;

/// The default thread local fallback handler.
pub static DEFAULT_THREAD_LOCAL_FALLBACK_HANDLER: ThreadLocalFallbackHandler =
    ThreadLocalFallbackHandler::DEFAULT;

impl_try_drop_strategy_for!(ThreadLocalFallbackHandler where Scope: ThreadLocalScope);

thread_local! {
    static FALLBACK_HANDLER: RefCell<Option<Box<dyn ThreadLocalTryDropStrategy>>> = RefCell::new(None);
    static LOCKED: RefCell<bool> = RefCell::new(false);
}

impl ThreadLocalDefinition for Fallback {
    const UNINITIALIZED_ERROR: &'static str =
        "the thread local fallback handler is not initialized yet";
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

/// A scope guard for the thread local fallback handler. This sets the thread local fallback handler
/// to the one specified for the duration of the scope.
pub type ScopeGuard = GenericScopeGuard<Fallback>;

/// A handy type alias for `Box<dyn ThreadLocalTryDropStrategy>`.
pub type BoxDynTryDropStrategy = Box<dyn ThreadLocalTryDropStrategy>;

thread_local_methods! {
    ThreadLocal = ThreadLocal;
    ScopeGuard = ScopeGuard;
    GenericStrategy = ThreadLocalTryDropStrategy;
    DynStrategy = BoxDynTryDropStrategy;
    feature = "ds-panic";

    /// Install a new fallback thread local handler.
    install;

    /// Install a new fallback thread local handler. Must be a dynamic trait object.
    install_dyn;

    /// Get a reference to the current fallback thread local handler.
    ///
    /// # Panics
    /// If the fallback thread local handler is not initialized yet, this function will panic.
    read;

    /// Try to get a reference to the current fallback thread local handler.
    ///
    /// # Errors
    /// If the fallback thread local handler is not initialized yet, this function will return an
    /// error.
    try_read;

    /// Get a reference to the current fallback thread local handler.
    ///
    /// If the fallback thread local handler is not initialized yet, this will set the fallback
    /// thread local handler to the default one.
    read_or_default;

    /// Get a mutable reference to the current fallback thread local handler.
    ///
    /// # Panics
    /// If the fallback thread local handler is not initialized yet, this function will panic.
    write;

    /// Try to get a mutable reference to the current fallback thread local handler.
    ///
    /// # Errors
    /// If the fallback thread local handler is not initialized yet, this function will return an
    /// error.
    try_write;

    /// Get a mutable reference to the current fallback thread local handler.
    ///
    /// If the fallback thread local handler is not initialized yet, this will set the fallback
    /// thread local handler to the default one.
    write_or_default;

    /// Uninstall the current fallback thread local handler.
    uninstall;

    /// Take the current fallback thread local handler, if there is any.
    take;

    /// Replace the current fallback thread local handler with a new one, returning the old one if
    /// any.
    replace;

    /// Replace the current fallback thread local handler with a new one, returning the old one if
    /// any. Must be a dynamic trait object.
    replace_dyn;

    /// Sets the fallback thread local handler to the specified one for the duration of the scope.
    scope;

    /// Sets the fallback thread local handler to the specified one for the duration of the scope.
    /// Must be a dynamic trait object.
    scope_dyn;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drop_strategies::{AdHocDropStrategy, AdHocFallibleDropStrategy, IntoAdHocDropStrategy, NoOpDropStrategy};
    use crate::handlers::{fallback, primary};
    use crate::test_utils::{ErrorsOnDrop, Fallible, FallibleDropStrategy};
    use crate::PureTryDrop;
    use anyhow::anyhow;
    use std::rc::Rc;

    #[test]
    #[should_panic]
    fn test_panic_on_uninit() {
        ThreadLocalFallbackHandler::on_uninit_panic().handle_error(anyhow!("test"))
    }

    #[test]
    fn test_flag_on_uninit() {
        let handler = ThreadLocalFallbackHandler::on_uninit_flag();
        assert!(
            !handler.last_drop_failed(),
            "last drop error handle failed but we haven't dropped anything yet"
        );
        handler.handle_error(anyhow!("test"));
        assert!(handler.last_drop_failed(), "last drop error handle didn't fail but we have dropped something while not initialized");
        install(NoOpDropStrategy);
        handler.handle_error(anyhow!("test"));
        assert!(
            !handler.last_drop_failed(),
            "last drop error handle failed but we have initialized"
        );
    }

    #[test]
    fn test_install() {
        let installed = Rc::new(RefCell::new(false));
        let i = Rc::clone(&installed);
        install((move |_| *i.borrow_mut() = true).into_drop_strategy());
        primary::thread_local::install(FallibleDropStrategy);
        drop(ErrorsOnDrop::<Fallible, _>::not_given().adapt());
        assert!(*installed.borrow(), "install didn't install");
    }

    #[test]
    fn test_install_dyn() {
        let installed = Rc::new(RefCell::new(false));
        let i = Rc::clone(&installed);
        install_dyn(Box::new(
            (move |_| *i.borrow_mut() = true).into_drop_strategy(),
        ));
        primary::thread_local::install(FallibleDropStrategy);
        drop(ErrorsOnDrop::<Fallible, _>::not_given().adapt());
        assert!(*installed.borrow(), "install_dyn didn't install");
    }

    #[test]
    #[should_panic(
        expected = "the thread local fallback handler is not initialized yet: UninitializedError(())"
    )]
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
    #[should_panic(
        expected = "the thread local fallback handler is not initialized yet: UninitializedError(())"
    )]
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
