//! Types and traits for the unreachable drop strategy, a.k.a. the drop strategy which you should
//! never use.
//!
//! The unreachable drop strategy does not get reexported in [`drop_strategies`](super). This is
//! intentional; we want this to be hidden away as much as possible.
mod private {
    pub trait Sealed {}
}

use crate::TryDropStrategy;
use core::marker::PhantomData;

/// How safe will the [`UnreachableDropStrategy`] be.
pub trait Safety: private::Sealed {}

/// Just panic when an error occurs.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
pub enum Safe {}
impl Safety for Safe {}
impl private::Sealed for Safe {}

/// Tell to the compiler that this branch never happens, a.k.a. call
/// [`core::hint::unreachable_unchecked`].
///
/// Note that when `debug_assertions` or the debug profile is used, this will just panic instead.
#[cfg(feature = "ds-unreachable-unsafe")]
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
pub enum Unsafe {}

#[cfg(feature = "ds-unreachable-unsafe")]
impl Safety for Unsafe {}

#[cfg(feature = "ds-unreachable-unsafe")]
impl private::Sealed for Unsafe {}

/// A try drop strategy whose error handling mechanism should never happen.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
pub struct UnreachableDropStrategy<S: Safety>(PhantomData<S>);

impl UnreachableDropStrategy<Safe> {
    /// Safe version of the unreachable drop strategy.
    pub const SAFE: Self = Self::safe();

    /// Create an unreachable drop strategy which just panics.
    pub const fn safe() -> Self {
        UnreachableDropStrategy(PhantomData)
    }
}

#[cfg(feature = "ds-unreachable-unsafe")]
impl UnreachableDropStrategy<Unsafe> {
    /// Unsafe version of the unreachable drop strategy.
    pub const UNSAFE: Self = Self::r#unsafe();

    /// Create an unreachable drop strategy which calls [`core::hint::unreachable_unchecked`]. Here
    /// be dragons!
    ///
    /// # Notes
    /// If debug assertions or the debug profile is used, this will just panic instead.
    ///
    /// While this function may be safe, the possible ramifications of continued use of this object
    /// as a drop strategy can eventually cause you to make undefined behavior if you aren't careful
    /// enough.
    pub const fn r#unsafe() -> Self {
        UnreachableDropStrategy(PhantomData)
    }
}

impl Default for UnreachableDropStrategy<Safe> {
    fn default() -> Self {
        Self::safe()
    }
}

impl TryDropStrategy for UnreachableDropStrategy<Safe> {
    fn handle_error(&self, error: crate::Error) {
        unreachable!("this error should not happen: {}", error)
    }
}

#[cfg(feature = "ds-unreachable-unsafe")]
impl TryDropStrategy for UnreachableDropStrategy<Unsafe> {
    fn handle_error(&self, error: crate::Error) {
        #[cfg(debug_assertions)]
        unreachable!(
            "panicking due to `debug_assertions` (debug profile), this error should not happen: {}",
            error
        );

        #[cfg(not(debug_assertions))]
        unsafe {
            core::hint::unreachable_unchecked()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::fallible;
    use super::*;

    #[test]
    #[should_panic(expected = "internal error: entered unreachable code: this error should not happen: this will always fail")]
    fn test_unreachable_drop_strategy_safe() {
        let unreachable = UnreachableDropStrategy::safe();
        crate::install_thread_local_handlers(unreachable, unreachable);
        drop(fallible());
    }

    #[test]
    #[should_panic(expected = "internal error: entered unreachable code: panicking due to `debug_assertions` (debug profile), this error should not happen: this will always fail")]
    #[cfg(debug_assertions)]
    fn test_unreachable_drop_strategy_unsafe_debug_assertions() {
        let unreachable = UnreachableDropStrategy::r#unsafe();
        crate::install_thread_local_handlers(unreachable, unreachable);
        drop(fallible());
    }

    // it is not possible to test the release version of `UnreachableDropStrategy<Unsafe>, as
    // executing its try drop handler will cause undefined behavior.
}