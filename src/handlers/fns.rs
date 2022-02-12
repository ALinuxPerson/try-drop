use std::boxed::Box;
use crate::{DynFallibleTryDropStrategy, TryDropStrategy};
use super::{fallback, primary};

#[cfg(feature = "global")]
use crate::{GlobalDynFallibleTryDropStrategy, GlobalTryDropStrategy};

/// This installs the primary and fallback global handlers.
#[cfg(feature = "global")]
pub fn install_global_handlers(
    primary: impl GlobalDynFallibleTryDropStrategy,
    fallback: impl GlobalTryDropStrategy,
) {
    install_global_handlers_dyn(Box::new(primary), Box::new(fallback))
}

/// This installs the primary and fallback global handlers. Must be a dynamic trait object.
#[cfg(feature = "global")]
pub fn install_global_handlers_dyn(
    primary: Box<dyn GlobalDynFallibleTryDropStrategy>,
    fallback: Box<dyn GlobalTryDropStrategy>,
) {
    primary::global::install_dyn(primary);
    fallback::global::install_dyn(fallback);
}

/// This uninstalls the primary and fallback global handlers.
#[cfg(feature = "global")]
pub fn uninstall_globally() {
    primary::global::uninstall();
    fallback::global::uninstall();
}

/// This installs the primary and fallback thread local handlers.
#[cfg(feature = "thread-local")]
pub fn install_thread_local_handlers(
    primary: impl DynFallibleTryDropStrategy + 'static,
    fallback: impl TryDropStrategy + 'static
) {
    install_thread_local_handlers_dyn(Box::new(primary), Box::new(fallback))
}

/// This installs the primary and fallback thread local handlers. Must be a dynamic trait
/// object.
#[cfg(feature = "thread-local")]
pub fn install_thread_local_handlers_dyn(
    primary: Box<dyn DynFallibleTryDropStrategy>,
    fallback: Box<dyn TryDropStrategy>,
) {
    primary::thread_local::install_dyn(primary);
    fallback::thread_local::install_dyn(fallback);
}

/// This installs the primary and fallback thread local handlers for this scope.
#[cfg(feature = "thread-local")]
pub fn install_thread_local_handlers_for_this_scope(
    primary: impl DynFallibleTryDropStrategy + 'static,
    fallback: impl TryDropStrategy + 'static,
) -> (primary::thread_local::ScopeGuard, fallback::thread_local::ScopeGuard) {
    install_thread_local_handlers_for_this_scope_dyn(Box::new(primary), Box::new(fallback))
}

/// This installs the primary and fallback thread local handlers for this scope. Must be a
/// dynamic trait object.
#[cfg(feature = "thread-local")]
pub fn install_thread_local_handlers_for_this_scope_dyn(
    primary: Box<dyn DynFallibleTryDropStrategy>,
    fallback: Box<dyn TryDropStrategy>,
) -> (primary::thread_local::ScopeGuard, fallback::thread_local::ScopeGuard) {
    (
        primary::thread_local::scope_dyn(primary),
        fallback::thread_local::scope_dyn(fallback),
    )
}

/// This uninstalls the primary and fallback thread local handlers.
#[cfg(feature = "thread-local")]
pub fn uninstall_for_thread() {
    primary::thread_local::uninstall();
    fallback::thread_local::uninstall();
}
