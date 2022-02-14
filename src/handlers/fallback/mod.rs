//! Manage the fallback handler.

use crate::handlers::common::Fallback;
use crate::handlers::common::proxy::TheGreatAbstracter;
use crate::handlers::on_uninit::UseDefaultOnUninit;

#[cfg(feature = "global")]
pub mod global;

#[cfg(feature = "thread-local")]
pub mod thread_local;

#[cfg(all(feature = "global", feature = "thread-local"))]
pub mod shim;

#[cfg(all(feature = "global", feature = "thread-local"))]
pub mod sshim;

mod handler;

mod private {
    pub trait Sealed {}
}

/// The default thing to do when the fallback handler is not initialized.
#[cfg(not(feature = "ds-panic"))]
pub type DefaultOnUninit = PanicOnUninit;

/// The default thing to do when the fallback handler is not initialized.
#[cfg(feature = "ds-panic")]
pub type DefaultOnUninit = UseDefaultOnUninit;

type Abstracter<S> = TheGreatAbstracter<Fallback, S>;
