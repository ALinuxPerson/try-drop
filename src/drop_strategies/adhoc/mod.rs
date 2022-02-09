#[cfg(feature = "ds-adhoc-mut")]
mod fn_mut;

#[cfg(feature = "ds-adhoc-mut")]
pub use fn_mut::*;

use crate::{FallibleTryDropStrategy, TryDropStrategy};
use core::marker::PhantomData;

/// A quick and dirty try drop strategy which uses a function.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
#[cfg_attr(feature = "shrinkwraprs", shrinkwrap(mutable))]
pub struct AdHocTryDropStrategy<F: Fn(crate::Error)>(pub F);

impl<F: Fn(crate::Error)> AdHocTryDropStrategy<F> {
    /// Take the inner function.
    #[cfg(feature = "shrinkwraprs")]
    pub fn take(this: Self) -> F {
        this.0
    }
}

impl<F: Fn(crate::Error)> TryDropStrategy for AdHocTryDropStrategy<F> {
    fn handle_error(&self, error: crate::Error) {
        self.0(error)
    }
}

impl<F: Fn(crate::Error)> From<F> for AdHocTryDropStrategy<F> {
    fn from(f: F) -> Self {
        AdHocTryDropStrategy(f)
    }
}

/// Signifies that this type can be converted into an [`AdHocTryDropStrategy`].
pub trait IntoAdHocTryDropStrategy: Fn(crate::Error) + Sized {
    /// Convert this type into an [`AdHocTryDropStrategy`].
    fn into_adhoc_try_drop_strategy(self) -> AdHocTryDropStrategy<Self> {
        AdHocTryDropStrategy(self)
    }
}

impl<T: Fn(crate::Error)> IntoAdHocTryDropStrategy for T {}

/// A quick and dirty fallible try drop strategy which uses a function.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
#[cfg_attr(feature = "shrinkwraprs", shrinkwrap(mutable))]
pub struct AdHocFallibleTryDropStrategy<F, E>
where
    F: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    /// The inner function.
    #[cfg_attr(feature = "shrinkwraprs", shrinkwrap(main_field))]
    pub f: F,

    _error: PhantomData<E>,
}

impl<F, E> AdHocFallibleTryDropStrategy<F, E>
where
    F: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    /// Create a new ad-hoc fallible try drop strategy.
    pub fn new(f: F) -> Self {
        Self {
            f,
            _error: PhantomData,
        }
    }

    /// Take the inner function.
    #[cfg(feature = "shrinkwraprs")]
    pub fn take(this: Self) -> F {
        this.f
    }
}

impl<F, E> FallibleTryDropStrategy for AdHocFallibleTryDropStrategy<F, E>
where
    F: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    type Error = E;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        (self.f)(error)
    }
}

impl<F, E> From<F> for AdHocFallibleTryDropStrategy<F, E>
where
    F: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    fn from(f: F) -> Self {
        Self::new(f)
    }
}

/// Signifies that this type can be converted into an [`AdHocFallibleTryDropStrategy`].
pub trait IntoAdHocFallibleTryDropStrategy:
    Fn(crate::Error) -> Result<(), Self::Error> + Sized
{
    /// The error type.
    type Error: Into<anyhow::Error>;

    /// Convert this type into an [`AdHocFallibleTryDropStrategy`].
    fn into_adhoc_fallible_try_drop_strategy(
        self,
    ) -> AdHocFallibleTryDropStrategy<Self, Self::Error> {
        AdHocFallibleTryDropStrategy::new(self)
    }
}
