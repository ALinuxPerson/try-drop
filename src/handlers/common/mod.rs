mod private {
    pub trait Sealed {}
}
pub mod shim;
pub mod thread_local;

use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

/// This error occurs when you attempt to use a scope guard in a nested scope.
///
/// # Examples
/// ```rust
/// {
///     let _guard = ScopeGuard::new(PanicDropStrategy::DEFAULT));
///     {
///         // this isn't allowed
///         let _guard = ScopeGuard::new(PanicDropStrategy::DEFAULT));
///     }
/// }
/// ```
#[cfg_attr(
    feature = "derives",
    derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
#[derive(Debug)]
pub struct NestedScopeError(pub(crate) ());

impl Error for NestedScopeError {}

impl fmt::Display for NestedScopeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("you cannot nest scope guards")
    }
}

pub trait Handler: private::Sealed {}

pub enum Primary {}
impl private::Sealed for Primary {}
impl Handler for Primary {}

pub enum Fallback {}
impl private::Sealed for Fallback {}
impl Handler for Fallback {}