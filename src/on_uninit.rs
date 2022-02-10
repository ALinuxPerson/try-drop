//! Types and traits on what to do when the global, thread local, or shim drop strategies is
//! uninitialized.
mod private {
    pub trait Sealed {}
}

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
