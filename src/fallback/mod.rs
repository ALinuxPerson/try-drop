//! Type and traits for fallback try drop strategies.

#[cfg(feature = "global")]
pub mod global;

#[cfg(feature = "thread-local")]
pub mod thread_local;

mod private {
    pub trait Sealed {}
}

use crate::on_uninit::OnUninit;
use core::sync::atomic::AtomicBool;

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
