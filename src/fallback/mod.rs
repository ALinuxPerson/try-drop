#[cfg(feature = "global")]
pub mod global;

use crate::{DynFallibleTryDropStrategy, FallibleTryDropStrategy, TryDropStrategy};
use anyhow::Error;

/// An error handler for drop strategies. If a struct implements [`TryDropStrategy`], it can also
/// be used as a [`FallbackDropStrategy`].
///
/// This **cannot** fail. However, if a failure somehow occurs, you must panic.
pub trait FallbackDropStrategy {
    fn handle_error_in_drop_strategy(&self, error: anyhow::Error);
}

impl<TDS: TryDropStrategy> FallbackDropStrategy for TDS {
    fn handle_error_in_drop_strategy(&self, error: Error) {
        self.handle_error(error)
    }
}

#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
pub struct FallbackDropStrategyRef<'a, T: FallbackDropStrategy>(pub &'a T);

impl<'a, T: FallbackDropStrategy> FallbackDropStrategy for FallbackDropStrategyRef<'a, T> {
    fn handle_error_in_drop_strategy(&self, error: anyhow::Error) {
        self.0.handle_error_in_drop_strategy(error)
    }
}

#[cfg(feature = "global")]
#[cfg(not(feature = "downcast-rs"))]
pub trait GlobalFallbackDropStrategy: crate::ThreadSafe + FallbackDropStrategy {}

#[cfg(feature = "global")]
#[cfg(feature = "downcast-rs")]
pub trait GlobalFallbackDropStrategy:
    crate::ThreadSafe + downcast_rs::DowncastSync + FallbackDropStrategy
{
}

#[cfg(feature = "global")]
#[cfg(feature = "downcast-rs")]
downcast_rs::impl_downcast!(sync GlobalFallbackDropStrategy);

#[cfg(feature = "global")]
impl<T: crate::ThreadSafe + FallbackDropStrategy> GlobalFallbackDropStrategy for T {}

#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct FallbackDropStrategyHandler<FDS, FTDS>
where
    FDS: FallbackDropStrategy,
    FTDS: FallibleTryDropStrategy,
{
    pub fallback_drop_strategy: FDS,
    pub fallible_try_drop_strategy: FTDS,
}

impl<FDS, FTDS> FallbackDropStrategyHandler<FDS, FTDS>
where
    FDS: FallbackDropStrategy,
    FTDS: FallibleTryDropStrategy,
{
    pub fn new(fallback_drop_strategy: FDS, fallible_try_drop_strategy: FTDS) -> Self {
        Self {
            fallback_drop_strategy,
            fallible_try_drop_strategy,
        }
    }
}

impl<FDS, FTDS> TryDropStrategy for FallbackDropStrategyHandler<FDS, FTDS>
where
    FDS: FallbackDropStrategy,
    FTDS: FallibleTryDropStrategy,
{
    fn handle_error(&self, error: anyhow::Error) {
        if let Err(error) = self.fallible_try_drop_strategy.dyn_try_handle_error(error) {
            self.fallback_drop_strategy
                .handle_error_in_drop_strategy(error)
        }
    }
}
