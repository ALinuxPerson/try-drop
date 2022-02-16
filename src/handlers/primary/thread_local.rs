//! Manage the thread local primary handler.

use super::{Abstracter, DefaultOnUninit};
use crate::handlers::common::handler::CommonHandler;
use crate::handlers::common::thread_local::{
    scope_guard::ScopeGuard as GenericScopeGuard, ThreadLocal as GenericThreadLocal,
    ThreadLocalDefinition,
};
use crate::handlers::common::Primary;
use crate::handlers::common::ThreadLocal as ThreadLocalScope;
use crate::handlers::on_uninit::{ErrorOnUninit, FlagOnUninit, PanicOnUninit};
use crate::handlers::uninit_error::UninitializedError;
use crate::FallibleTryDropStrategy;
use std::boxed::Box;
use std::cell::RefCell;

use std::thread::LocalKey;
use std::{convert, thread_local};

#[cfg(feature = "ds-write")]
use crate::handlers::common::thread_local::DefaultThreadLocalDefinition;

#[cfg(feature = "ds-write")]
use crate::handlers::on_uninit::UseDefaultOnUninit;

/// A primary handler that uses the thread local scope.
pub type ThreadLocalPrimaryHandler<OU = DefaultOnUninit> =
    CommonHandler<OU, ThreadLocalScope, Primary>;

/// The default thread local primary handler.
pub static DEFAULT_THREAD_LOCAL_PRIMARY_HANDLER: ThreadLocalPrimaryHandler =
    ThreadLocalPrimaryHandler::DEFAULT;

impl_fallible_try_drop_strategy_for!(ThreadLocalPrimaryHandler
where
    Scope: ThreadLocalScope,
    Definition: ThreadLocalDefinition
);

thread_local! {
    static PRIMARY_HANDLER: RefCell<Option<Box<dyn ThreadLocalFallibleTryDropStrategy>>> = RefCell::new(None);
    static LOCKED: RefCell<bool> = RefCell::new(false);
}

impl ThreadLocalDefinition for Primary {
    const UNINITIALIZED_ERROR: &'static str =
        "the thread local primary handler is not initialized yet";
    const DYN: &'static str = "ThreadLocalFallibleTryDropStrategy";
    type ThreadLocal = Box<dyn ThreadLocalFallibleTryDropStrategy>;

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

impl<T: ThreadLocalFallibleTryDropStrategy> From<T>
    for Box<dyn ThreadLocalFallibleTryDropStrategy>
{
    fn from(strategy: T) -> Self {
        Box::new(strategy)
    }
}

type ThreadLocal = GenericThreadLocal<Primary>;

/// A scope guard for the thread local primary handler. It is used to set the thread local primary
/// handler for the duration of the scope.
pub type ScopeGuard = GenericScopeGuard<Primary>;

/// Handy type alias to `Box<dyn ThreadLocalFallibleTryDropStrategy>`.
pub type BoxDynFallibleTryDropStrategy = Box<dyn ThreadLocalFallibleTryDropStrategy>;

thread_local_methods! {
    ThreadLocal = ThreadLocal;
    ScopeGuard = ScopeGuard;
    GenericStrategy = ThreadLocalFallibleTryDropStrategy;
    DynStrategy = BoxDynFallibleTryDropStrategy;
    feature = "ds-write";

    /// Install a new thread local primary handler.
    install;

    /// Install a new thread local primary handler. Must be a dynamic trait object.
    install_dyn;

    /// Get a reference to the current thread local primary handler.
    ///
    /// # Panics
    /// If the thread local primary handler is not initialized yet, this function will panic.
    read;

    /// Try and get a reference to the current thread local primary handler.
    ///
    /// # Errors
    /// If the thread local primary handler is not initialized yet, this function will return an
    /// error.
    try_read;

    /// Get a reference to the current thread local primary handler.
    ///
    /// If the current thread local primary handler is not initialized yet, this function will
    /// set it to the default primary handler.
    read_or_default;

    /// Get a mutable reference to the current thread local primary handler.
    ///
    /// # Panics
    /// If the thread local primary handler is not initialized yet, this function will panic.
    write;

    /// Try and get a mutable reference to the current thread local primary handler.
    ///
    /// # Errors
    /// If the thread local primary handler is not initialized yet, this function will return an
    try_write;

    /// Get a mutable reference to the current thread local primary handler.
    ///
    /// If the current thread local primary handler is not initialized yet, this function will
    /// set it to the default primary handler.
    write_or_default;

    /// Uninstall the current thread local primary handler.
    uninstall;

    /// Take the current thread local primary handler, if there is any initalized.
    take;

    /// Replace the current thread local primary handler with the given one, returning the old one
    /// if any.
    replace;

    /// Replace the current thread local primary handler with the given one, returning the old one
    /// if any. Must be a dynamic trait object.
    replace_dyn;

    /// Sets the thread local primary handler to the given one for the duration of the given scope.
    /// For more advanced usage, see the [`ScopeGuard`] type.
    scope;

    /// Sets the thread local primary handler to the given one for the duration of the given scope.
    /// For more advanced usage, see the [`ScopeGuard`] type. Must be a dynamic trait object.
    scope_dyn;
}
