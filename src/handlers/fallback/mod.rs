//! Manage the fallback handler.

#[cfg(feature = "global")]
pub mod global;

#[cfg(feature = "thread-local")]
pub mod thread_local;

#[cfg(all(feature = "global", feature = "thread-local"))]
pub mod shim;

mod handler;

mod private {
    pub trait Sealed {}
}
