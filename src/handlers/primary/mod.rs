//! Manage the primary handler.

#[cfg(feature = "global")]
pub mod global;

#[cfg(feature = "thread-local")]
pub mod thread_local;

pub mod handler;

#[cfg(all(feature = "global", feature = "thread-local"))]
pub mod shim;

pub mod sshim;
