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

    impl OnUninit for UseDefaultOnUninit {}
    impl private::Sealed for UseDefaultOnUninit {}
}

#[cfg(any(feature = "ds-write", feature = "ds-panic"))]
pub use use_default::*;

/// What to do when the global, thread local, or shim drop strategies is uninitialized.
pub trait OnUninit: private::Sealed {}

/// Just error on the drop strategy if uninitialized.
pub enum ErrorOnUninit {}

impl OnUninit for ErrorOnUninit {}
impl private::Sealed for ErrorOnUninit {}

/// Panic on the drop strategy if uninitialized.
pub enum PanicOnUninit {}

impl OnUninit for PanicOnUninit {}
impl private::Sealed for PanicOnUninit {}
