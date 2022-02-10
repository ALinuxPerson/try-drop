//! Type and traits for fallback try drop strategies.

#[cfg(feature = "global")]
pub mod global;

#[cfg(feature = "thread-local")]
pub mod thread_local;

#[cfg(all(feature = "global", feature = "thread-local"))]
pub mod shim;

use crate::{DynFallibleTryDropStrategy, FallibleTryDropStrategy, TryDropStrategy};
use anyhow::Error;

/// An error handler for drop strategies. If a struct implements [`TryDropStrategy`], it can also
/// be used as a [`FallbackTryDropStrategy`]. This **cannot** fail.
pub trait FallbackTryDropStrategy {
    /// Handle an error in a drop strategy.
    fn handle_error_in_strategy(&self, error: anyhow::Error);
}

impl<TDS: TryDropStrategy> FallbackTryDropStrategy for TDS {
    fn handle_error_in_strategy(&self, error: Error) {
        self.handle_error(error)
    }
}

/// A reference to a type which implements [`FallbackTryDropStrategy`]. Used as a workaround for
/// implementing [`FallbackTryDropStrategy`] on references.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
pub struct FallbackTryDropStrategyRef<'a, T: FallbackTryDropStrategy>(pub &'a T);

impl<'a, T: FallbackTryDropStrategy> FallbackTryDropStrategy for FallbackTryDropStrategyRef<'a, T> {
    fn handle_error_in_strategy(&self, error: anyhow::Error) {
        self.0.handle_error_in_strategy(error)
    }
}

#[cfg(feature = "global")]
#[cfg(not(feature = "downcast-rs"))]
pub trait GlobalFallbackTryDropStrategy: crate::ThreadSafe + FallbackTryDropStrategy {}

/// Signifies that a type is try drop strategy which can be used as a fallback, and can also be used
/// as the global fallback try drop strategy.
#[cfg(feature = "global")]
#[cfg(feature = "downcast-rs")]
pub trait GlobalFallbackTryDropStrategy:
    crate::ThreadSafe + downcast_rs::DowncastSync + FallbackTryDropStrategy
{
}

#[cfg(feature = "global")]
#[cfg(feature = "downcast-rs")]
downcast_rs::impl_downcast!(sync GlobalFallbackTryDropStrategy);

#[cfg(feature = "global")]
impl<T: crate::ThreadSafe + FallbackTryDropStrategy> GlobalFallbackTryDropStrategy for T {}

/// A type which chains two try drop strategies together, one of which may fail and if so, will be
/// redirected to the fallback, infallible try drop strategy.
///
/// This type implements [`TryDropStrategy`] because, as said before, any and all errors in the
/// fallible try drop strategy will be redirected to the fallback, which can never fail.
#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct FallbackTryDropStrategyHandler<FDS, FTDS>
where
    FDS: FallbackTryDropStrategy,
    FTDS: FallibleTryDropStrategy,
{
    /// The fallback try drop strategy. This will be called if the first try drop strategy fails and
    /// is a last resort to recovering sanely.
    pub fallback_try_drop_strategy: FDS,

    /// The try drop strategy which may fail. This will be called first.
    pub fallible_try_drop_strategy: FTDS,
}

impl<FDS, FTDS> FallbackTryDropStrategyHandler<FDS, FTDS>
where
    FDS: FallbackTryDropStrategy,
    FTDS: FallibleTryDropStrategy,
{
    /// Create a new fallback try drop strategy handler.
    pub fn new(fallback_try_drop_strategy: FDS, fallible_try_drop_strategy: FTDS) -> Self {
        Self {
            fallback_try_drop_strategy,
            fallible_try_drop_strategy,
        }
    }
}

impl<FDS, FTDS> TryDropStrategy for FallbackTryDropStrategyHandler<FDS, FTDS>
where
    FDS: FallbackTryDropStrategy,
    FTDS: FallibleTryDropStrategy,
{
    fn handle_error(&self, error: anyhow::Error) {
        if let Err(error) = self.fallible_try_drop_strategy.dyn_try_handle_error(error) {
            self.fallback_try_drop_strategy
                .handle_error_in_strategy(error)
        }
    }
}
