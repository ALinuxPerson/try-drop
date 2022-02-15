//! Manage the primary and fallback handlers and their scopes.

#[macro_use]
mod common;

pub mod fallback;
pub(crate) mod fns;
pub mod primary;

#[cfg(any(feature = "global", feature = "thread-local"))]
pub mod on_uninit;

#[cfg(any(feature = "global", feature = "thread-local"))]
mod uninit_error;

#[cfg(any(feature = "global", feature = "thread-local"))]
pub use uninit_error::UninitializedError;

pub use fns::*;

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub use primary::global::GlobalPrimaryHandler as PrimaryHandler;

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub use primary::global::DEFAULT_GLOBAL_PRIMARY_HANDLER as DEFAULT_PRIMARY_HANDLER;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub use primary::thread_local::ThreadLocalPrimaryHandler as PrimaryHandler;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub use primary::thread_local::DEFAULT_THREAD_LOCAL_PRIMARY_HANDLER as DEFAULT_PRIMARY_HANDLER;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub use primary::shim::ShimPrimaryHandler as PrimaryHandler;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub use primary::shim::DEFAULT_SHIM_PRIMARY_HANDLER as DEFAULT_PRIMARY_HANDLER;

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub use fallback::global::GlobalFallbackHandler as FallbackHandler;

#[cfg(all(feature = "global", not(feature = "thread-local")))]
pub use fallback::global::DEFAULT_GLOBAL_FALLBACK_HANDLER as DEFAULT_FALLBACK_HANDLER;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub use fallback::thread_local::ThreadLocalFallbackHandler as FallbackHandler;

#[cfg(all(feature = "thread-local", not(feature = "global")))]
pub use fallback::thread_local::DEFAULT_THREAD_LOCAL_FALLBACK_HANDLER as DEFAULT_FALLBACK_HANDLER;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub use fallback::shim::ShimFallbackHandler as FallbackHandler;

#[cfg(all(feature = "thread-local", feature = "global"))]
pub use fallback::shim::DEFAULT_SHIM_FALLBACK_HANDLER as DEFAULT_FALLBACK_HANDLER;
