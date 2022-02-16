#[cfg(feature = "ds-adhoc-mut")]
mod fn_mut;

#[cfg(feature = "ds-adhoc-mut")]
pub use fn_mut::*;

use crate::{FallibleTryDropStrategy, TryDropStrategy};

/// A quick and dirty drop strategy which uses a function.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
#[cfg_attr(feature = "shrinkwraprs", shrinkwrap(mutable))]
pub struct AdHocDropStrategy<F: Fn(crate::Error)>(pub F);

impl<F: Fn(crate::Error)> AdHocDropStrategy<F> {
    /// Take the inner function.
    #[cfg(feature = "shrinkwraprs")]
    pub fn take(this: Self) -> F {
        this.0
    }
}

impl<F: Fn(crate::Error)> TryDropStrategy for AdHocDropStrategy<F> {
    fn handle_error(&self, error: crate::Error) {
        self.0(error)
    }
}

impl<F: Fn(crate::Error)> From<F> for AdHocDropStrategy<F> {
    fn from(f: F) -> Self {
        AdHocDropStrategy(f)
    }
}

/// Signifies that this type can be converted into an [`AdHocDropStrategy`].
pub trait IntoAdHocDropStrategy: Fn(crate::Error) + Sized {
    /// Convert this type into an [`AdHocDropStrategy`].
    fn into_drop_strategy(self) -> AdHocDropStrategy<Self> {
        AdHocDropStrategy(self)
    }
}

impl<T: Fn(crate::Error)> IntoAdHocDropStrategy for T {}

/// A quick and dirty fallible drop strategy which uses a function.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
#[cfg_attr(feature = "shrinkwraprs", shrinkwrap(mutable))]
pub struct AdHocFallibleDropStrategy<F, E>(pub F)
where
    F: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>;

impl<F, E> AdHocFallibleDropStrategy<F, E>
where
    F: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    /// Take the inner function.
    #[cfg(feature = "shrinkwraprs")]
    pub fn take(this: Self) -> F {
        this.0
    }
}

impl<F, E> FallibleTryDropStrategy for AdHocFallibleDropStrategy<F, E>
where
    F: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    type Error = E;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        (self.0)(error)
    }
}

impl<F, E> From<F> for AdHocFallibleDropStrategy<F, E>
where
    F: Fn(crate::Error) -> Result<(), E>,
    E: Into<anyhow::Error>,
{
    fn from(f: F) -> Self {
        Self(f)
    }
}

/// Signifies that this type can be converted into an [`AdHocFallibleDropStrategy`].
pub trait IntoAdHocFallibleDropStrategy:
    Fn(crate::Error) -> Result<(), Self::Error> + Sized
{
    /// The error type.
    type Error: Into<anyhow::Error>;

    /// Convert this type into an [`AdHocFallibleDropStrategy`].
    fn into_drop_strategy(
        self,
    ) -> AdHocFallibleDropStrategy<Self, Self::Error> {
        AdHocFallibleDropStrategy(self)
    }
}
