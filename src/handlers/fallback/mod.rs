//! Manage the fallback handler.

#[cfg(feature = "global")]
pub mod global;

#[cfg(feature = "thread-local")]
pub mod thread_local;

#[cfg(all(feature = "global", feature = "thread-local"))]
pub mod shim;

#[cfg(all(feature = "global", feature = "thread-local"))]
pub mod sshim;

mod private {
    pub trait Sealed {}
}

use std::marker::PhantomData;
use crate::handlers::common::{Fallback, Scope};
use crate::handlers::common::handler::CommonHandler;
use crate::handlers::common::proxy::TheGreatAbstracter;
use crate::handlers::on_uninit::UseDefaultOnUninit;

/// The default thing to do when the fallback handler is not initialized.
#[cfg(not(feature = "ds-panic"))]
pub type DefaultOnUninit = PanicOnUninit;

/// The default thing to do when the fallback handler is not initialized.
#[cfg(feature = "ds-panic")]
pub type DefaultOnUninit = UseDefaultOnUninit;

type Abstracter<S> = TheGreatAbstracter<Fallback, S>;

impl<S: Scope> CommonHandler<DefaultOnUninit, S, Fallback> {
    pub const DEFAULT: Self = Self {
        extra_data: (),
        _scope: PhantomData,
    };
}

impl<S: Scope> Default for CommonHandler<DefaultOnUninit, S, Fallback> {
    fn default() -> Self {
        Self::DEFAULT
    }
}
