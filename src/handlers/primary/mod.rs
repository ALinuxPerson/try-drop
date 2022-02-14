//! Manage the primary handler.

use std::marker::PhantomData;
use crate::handlers::common::proxy::TheGreatAbstracter;
use crate::handlers::common::{Primary, Scope};
use crate::handlers::common::handler::CommonHandler;
use crate::handlers::on_uninit::{ErrorOnUninit, UseDefaultOnUninit};

#[cfg(feature = "global")]
pub mod global;

#[cfg(feature = "thread-local")]
pub mod thread_local;

#[cfg(all(feature = "global", feature = "thread-local"))]
pub mod shim;

/// The default thing to do when the primary handler is uninitialized, that is
/// to panic.
#[cfg(not(feature = "ds-write"))]
pub type DefaultOnUninit = PanicOnUninit;

/// The default thing to do when the primary handler is uninitialized, that is
/// to use the default strategy. Note that this mutates the primary handler.
#[cfg(feature = "ds-write")]
pub type DefaultOnUninit = UseDefaultOnUninit;

type Abstracter<S> = TheGreatAbstracter<Primary, S>;

impl<S: Scope> CommonHandler<ErrorOnUninit, S, Primary> {
    pub const ON_UNINIT_ERROR: Self = Self {
        extra_data: (),
        _scope: PhantomData,
    };

    pub fn error_on_uninit() -> Self {
        Self::ON_UNINIT_ERROR
    }
}

impl<S: Scope> CommonHandler<DefaultOnUninit, S, Primary> {
    pub const DEFAULT: Self = Self {
        extra_data: (),
        _scope: PhantomData,
    };
}
