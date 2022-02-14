//! Manage the primary handler.

#[cfg(feature = "global")]
pub mod global;

#[cfg(feature = "thread-local")]
pub mod thread_local;

#[cfg(feature = "thread-local")]
pub mod tthread_local;

#[cfg(all(feature = "global", feature = "thread-local"))]
pub mod shim;
