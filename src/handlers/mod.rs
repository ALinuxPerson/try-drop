//! Manage the primary and fallback handlers and their scopes.

pub mod primary;
pub mod fallback;
mod common;
mod shim;
#[cfg(any(feature = "global", feature = "thread-local"))]
pub mod on_uninit;
#[cfg(any(feature = "global", feature = "thread-local"))]
pub mod uninit_error;

pub(crate) mod fns {
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
}

pub use fns::*;

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub use primary::global::GlobalPrimaryDropStrategy as PrimaryDropStrategy;

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub use primary::global::DEFAULT_GLOBAL_PRIMARY_DROP_STRATEGY as DEFAULT_PRIMARY_DROP_STRATEGY;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub use primary::thread_local::ThreadLocalPrimaryTryDropStrategy as PrimaryDropStrategy;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub use primary::thread_local::DEFAULT_THREAD_LOCAL_PRIMARY_DROP_STRATEGY as DEFAULT_PRIMARY_DROP_STRATEGY;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub use primary::shim::ShimPrimaryDropStrategy as PrimaryDropStrategy;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub use primary::shim::DEFAULT_SHIM_PRIMARY_DROP_STRATEGY as DEFAULT_PRIMARY_DROP_STRATEGY;

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub use fallback::global::GlobalFallbackDropStrategy as FallbackDropStrategy;

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub use fallback::global::DEFAULT_GLOBAL_FALLBACK_STRATEGY as DEFAULT_FALLBACK_DROP_STRATEGY;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub use fallback::thread_local::ThreadLocalFallbackDropStrategy as FallbackDropStrategy;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub use fallback::thread_local::DEFAULT_THREAD_LOCAL_FALLBACK_STRATEGY as DEFAULT_FALLBACK_DROP_STRATEGY;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub use fallback::shim::ShimFallbackDropStrategy as FallbackDropStrategy;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub use fallback::shim::DEFAULT_SHIM_FALLBACK_DROP_STRATEGY as DEFAULT_FALLBACK_DROP_STRATEGY;

