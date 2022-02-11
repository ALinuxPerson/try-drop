//! Type and traits for fallback try drop strategies.

#[cfg(feature = "global")]
pub mod global;

#[cfg(feature = "thread-local")]
pub mod thread_local;

mod private {
    pub trait Sealed {}
}

use crate::on_uninit::OnUninit;
use crate::{DynFallibleTryDropStrategy, FallibleTryDropStrategy, TryDropStrategy};
use anyhow::Error;
use std::sync::atomic::AtomicBool;

// /// An error handler for drop strategies. If a struct implements [`TryDropStrategy`], it can also
// /// be used as a [`FallbackTryDropStrategy`]. This **cannot** fail.
// pub trait FallbackTryDropStrategy {
//     /// Handle an error in a drop strategy.
//     fn handle_error_in_strategy(&self, error: anyhow::Error);
// }
//
// impl<TDS: TryDropStrategy> FallbackTryDropStrategy for TDS {
//     fn handle_error_in_strategy(&self, error: Error) {
//         self.handle_error(error)
//     }
// }
//
// #[cfg(feature = "global")]
// #[cfg(not(feature = "downcast-rs"))]
// pub trait GlobalFallbackTryDropStrategy: crate::ThreadSafe + FallbackTryDropStrategy {}
//
// /// Signifies that a type is try drop strategy which can be used as a fallback, and can also be used
// /// as the global fallback try drop strategy.
// #[cfg(feature = "global")]
// #[cfg(feature = "downcast-rs")]
// pub trait GlobalFallbackTryDropStrategy:
//     crate::ThreadSafe + downcast_rs::DowncastSync + FallbackTryDropStrategy
// {
// }
//
// #[cfg(feature = "global")]
// #[cfg(feature = "downcast-rs")]
// downcast_rs::impl_downcast!(sync GlobalFallbackTryDropStrategy);
//
// #[cfg(feature = "global")]
// impl<T: crate::ThreadSafe + FallbackTryDropStrategy> GlobalFallbackTryDropStrategy for T {}

pub trait OnUninitFallback: private::Sealed {
    type ExtraData;
}

impl<OU: OnUninit> OnUninitFallback for OU {
    type ExtraData = ();
}

impl<OU: OnUninit> private::Sealed for OU {}

pub enum FlagOnUninit {}

impl OnUninitFallback for FlagOnUninit {
    type ExtraData = AtomicBool;
}
impl private::Sealed for FlagOnUninit {}
