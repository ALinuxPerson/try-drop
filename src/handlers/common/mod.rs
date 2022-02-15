mod private {
    pub trait Sealed {}
}
pub mod shim;

#[macro_use]
pub mod thread_local;

#[macro_use]
pub mod global;

pub mod handler;
pub mod proxy;

use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use crate::handlers::UninitializedError;

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

pub trait Scope: private::Sealed {}

pub enum Global {}
impl private::Sealed for Global {}
impl Scope for Global {}

pub enum ThreadLocal {}
impl private::Sealed for ThreadLocal {}
impl Scope for ThreadLocal {}

pub trait ScopeAccessor {
    type Access;

    fn install(strategy: impl Into<Self::Access>);
    fn install_dyn(strategy: Self::Access);
    fn try_read<R>(f: impl FnOnce(&Self::Access) -> R) -> Result<R, UninitializedError>;
    fn read<R>(f: impl FnOnce(&Self::Access) -> R) -> R;
    fn try_write<R>(f: impl FnOnce(&mut Self::Access) -> R) -> Result<R, UninitializedError>;
    fn write<R>(f: impl FnOnce(&mut Self::Access) -> R) -> R;
    fn uninstall();
}

pub trait DefaultScopeAccessor: ScopeAccessor {
    fn read_or_default<R>(f: impl FnOnce(&Self::Access) -> R) -> R;
    fn write_or_default<R>(f: impl FnOnce(&mut Self::Access) -> R) -> R;
}
