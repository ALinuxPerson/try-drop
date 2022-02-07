#[cfg(feature = "global")]
pub mod global;

use crate::{DynFallibleTryDropStrategy, FallibleTryDropStrategy, TryDropStrategy};
use anyhow::Error;

/// An error handler for drop strategies. If a struct implements [`TryDropStrategy`], it can also
/// be used as a [`FallbackTryDropStrategy`].
///
/// This **cannot** fail. However, if a failure somehow occurs, you must panic.
pub trait FallbackTryDropStrategy {
    fn handle_error_in_strategy(&self, error: anyhow::Error);
}

impl<TDS: TryDropStrategy> FallbackTryDropStrategy for TDS {
    fn handle_error_in_strategy(&self, error: Error) {
        self.handle_error(error)
    }
}

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

#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct FallbackTryDropStrategyHandler<FDS, FTDS>
where
    FDS: FallbackTryDropStrategy,
    FTDS: FallibleTryDropStrategy,
{
    pub fallback_try_drop_strategy: FDS,
    pub fallible_try_drop_strategy: FTDS,
}

impl<FDS, FTDS> FallbackTryDropStrategyHandler<FDS, FTDS>
where
    FDS: FallbackTryDropStrategy,
    FTDS: FallibleTryDropStrategy,
{
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
