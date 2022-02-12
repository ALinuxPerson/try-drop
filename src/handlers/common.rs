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
