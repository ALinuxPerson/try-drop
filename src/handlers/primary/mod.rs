//! Manage the primary handler.

#[cfg(feature = "global")]
pub mod global;

#[cfg(feature = "global")]
pub mod gglobal;

#[cfg(feature = "thread-local")]
pub mod thread_local;

#[cfg(all(feature = "global", feature = "thread-local"))]
pub mod shim;
