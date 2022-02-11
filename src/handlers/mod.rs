pub mod primary;
pub mod fallback;
mod common;
mod shim;
pub(crate) mod fns {
    #[cfg(feature = "global")]
    pub fn install_global_handlers(
        primary: impl GlobalDynFallibleTryDropStrategy,
        fallback: impl GlobalTryDropStrategy,
    ) {
        install_global_handlers_dyn(Box::new(primary), Box::new(fallback))
    }

    #[cfg(feature = "global")]
    pub fn install_global_handlers_dyn(
        primary: Box<dyn GlobalDynFallibleTryDropStrategy>,
        fallback: Box<dyn GlobalTryDropStrategy>,
    ) {
        primary::global::install_dyn(primary);
        fallback::global::install_dyn(fallback);
    }

    #[cfg(feature = "global")]
    pub fn uninstall_globally() {
        primary::global::uninstall();
        fallback::global::uninstall();
    }

    #[cfg(feature = "thread-local")]
    pub fn install_thread_local_handlers(
        primary: impl DynFallibleTryDropStrategy + 'static,
        fallback: impl TryDropStrategy + 'static
    ) {
        install_thread_local_handlers_dyn(Box::new(primary), Box::new(fallback))
    }

    #[cfg(feature = "thread-local")]
    pub fn install_thread_local_handlers_dyn(
        primary: Box<dyn DynFallibleTryDropStrategy>,
        fallback: Box<dyn TryDropStrategy>,
    ) {
        primary::thread_local::install_dyn(primary);
        fallback::thread_local::install_dyn(fallback);
    }

    #[cfg(feature = "thread-local")]
    pub fn uninstall_for_thread() {
        primary::thread_local::uninstall();
        fallback::thread_local::uninstall();
    }
}

pub use fns::*;
use std::boxed::Box;
use crate::{DynFallibleTryDropStrategy, GlobalDynFallibleTryDropStrategy, GlobalTryDropStrategy, TryDropStrategy};

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub type PrimaryDropStrategy = primary::global::GlobalPrimaryDropStrategy;

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub use primary::global::DEFAULT_GLOBAL_PRIMARY_DROP_STRATEGY as DEFAULT_PRIMARY_DROP_STRATEGY;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub type PrimaryDropStrategy = primary::thread_local::ThreadLocalPrimaryTryDropStrategy;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub use primary::thread_local::DEFAULT_THREAD_LOCAL_PRIMARY_DROP_STRATEGY as DEFAULT_PRIMARY_DROP_STRATEGY;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub type PrimaryDropStrategy = primary::shim::ShimPrimaryDropStrategy;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub use primary::shim::DEFAULT_SHIM_PRIMARY_DROP_STRATEGY as DEFAULT_PRIMARY_DROP_STRATEGY;

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub type FallbackDropStrategy = fallback::global::GlobalFallbackDropStrategy;

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub use fallback::global::DEFAULT_GLOBAL_FALLBACK_STRATEGY as DEFAULT_FALLBACK_DROP_STRATEGY;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub type FallbackDropStrategy = fallback::thread_local::ThreadLocalFallbackDropStrategy;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub use fallback::thread_local::DEFAULT_THREAD_LOCAL_FALLBACK_STRATEGY as DEFAULT_FALLBACK_DROP_STRATEGY;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub type FallbackDropStrategy = fallback::shim::ShimFallbackDropStrategy;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub use fallback::shim::DEFAULT_SHIM_FALLBACK_DROP_STRATEGY as DEFAULT_FALLBACK_DROP_STRATEGY;

