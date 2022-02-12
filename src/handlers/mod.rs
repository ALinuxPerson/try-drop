//! Manage the primary and fallback handlers and their scopes.

mod common;
pub mod fallback;
pub(crate) mod fns;
pub mod primary;
mod shim;

#[cfg(any(feature = "global", feature = "thread-local"))]
pub mod on_uninit;

#[cfg(any(feature = "global", feature = "thread-local"))]
mod uninit_error;

#[cfg(any(feature = "global", feature = "thread-local"))]
pub use uninit_error::UninitializedError;

pub use fns::*;

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub use primary::global::GlobalPrimaryDropStrategy as PrimaryDropStrategy;

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub use primary::global::DEFAULT_GLOBAL_PRIMARY_DROP_STRATEGY as DEFAULT_PRIMARY_DROP_STRATEGY;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub use primary::thread_local::ThreadLocalPrimaryHandler as PrimaryDropStrategy;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub use primary::thread_local::DEFAULT_THREAD_LOCAL_PRIMARY_DROP_STRATEGY as DEFAULT_PRIMARY_DROP_STRATEGY;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub use primary::shim::ShimPrimaryDropStrategy as PrimaryDropStrategy;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub use primary::shim::DEFAULT_SHIM_PRIMARY_DROP_STRATEGY as DEFAULT_PRIMARY_DROP_STRATEGY;

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub use fallback::global::GlobalFallbackHandler as FallbackDropStrategy;

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub use fallback::global::DEFAULT_GLOBAL_FALLBACK_HANDLER as DEFAULT_FALLBACK_DROP_STRATEGY;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub use fallback::thread_local::ThreadLocalFallbackHandler as FallbackDropStrategy;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub use fallback::thread_local::DEFAULT_THREAD_LOCAL_FALLBACK_HANDLER as DEFAULT_FALLBACK_DROP_STRATEGY;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub use fallback::shim::ShimFallbackHandler as FallbackDropStrategy;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub use fallback::shim::DEFAULT_SHIM_FALLBACK_HANDLER as DEFAULT_FALLBACK_DROP_STRATEGY;
