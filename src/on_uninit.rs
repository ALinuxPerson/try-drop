//! Types and traits on what to do when the global, thread local, or shim drop strategies is
//! uninitialized.
mod private {
    pub trait Sealed {}
}

#[cfg(any(feature = "ds-write", feature = "ds-panic"))]
mod use_default {
    use super::*;

    /// Use the default drop strategy if uninitialized
    pub enum UseDefaultOnUninit {}

    impl OnUninit for UseDefaultOnUninit {
        type ExtraData = ();
    }
    impl private::Sealed for UseDefaultOnUninit {}
}

use core::sync::atomic::AtomicBool;
#[cfg(any(feature = "ds-write", feature = "ds-panic"))]
pub use use_default::*;

/// What to do when the global, thread local, or shim drop strategies is uninitialized.
pub trait OnUninit: private::Sealed {
    /// Any extra data that this type may neee.
    type ExtraData;
}

/// Just error on the drop strategy if uninitialized.
pub enum ErrorOnUninit {}

impl OnUninit for ErrorOnUninit {
    type ExtraData = ();
}
impl private::Sealed for ErrorOnUninit {}

/// Panic on the drop strategy if uninitialized.
pub enum PanicOnUninit {}

impl OnUninit for PanicOnUninit {
    type ExtraData = ();
}
impl private::Sealed for PanicOnUninit {}

/// Does nothing if uninitialized.
pub enum DoNothingOnUninit {}

impl OnUninit for DoNothingOnUninit {
    type ExtraData = ();
}
impl private::Sealed for DoNothingOnUninit {}

/// Sets an internal flag if uninitialized.
pub enum FlagOnUninit {}

impl OnUninit for FlagOnUninit {
    type ExtraData = AtomicBool;
}
impl private::Sealed for FlagOnUninit {}