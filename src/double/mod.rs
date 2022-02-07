#[cfg(feature = "global")]
pub mod global;

use crate::{DynFallibleTryDropStrategy, FallibleTryDropStrategy, TryDropStrategy};
use anyhow::Error;

/// An error handler for drop strategies. If a struct implements [`TryDropStrategy`], it can also
/// be used as a [`DoubleDropStrategy`].
///
/// This **cannot** fail. However, if a failure somehow occurs, you must panic.
pub trait DoubleDropStrategy {
    fn handle_error_in_drop_strategy(&self, error: anyhow::Error);
}

impl<TDS: TryDropStrategy> DoubleDropStrategy for TDS {
    fn handle_error_in_drop_strategy(&self, error: Error) {
        self.handle_error(error)
    }
}

#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
pub struct DoubleDropStrategyRef<'a, T: DoubleDropStrategy>(pub &'a T);

impl<'a, T: DoubleDropStrategy> DoubleDropStrategy for DoubleDropStrategyRef<'a, T> {
    fn handle_error_in_drop_strategy(&self, error: anyhow::Error) {
        self.0.handle_error_in_drop_strategy(error)
    }
}

#[cfg(feature = "global")]
#[cfg(not(feature = "downcast-rs"))]
pub trait GlobalDoubleDropStrategy: crate::ThreadSafe + DoubleDropStrategy {}

#[cfg(feature = "global")]
#[cfg(feature = "downcast-rs")]
pub trait GlobalDoubleDropStrategy:
    crate::ThreadSafe + downcast_rs::DowncastSync + DoubleDropStrategy
{
}

#[cfg(feature = "global")]
#[cfg(feature = "downcast-rs")]
downcast_rs::impl_downcast!(sync GlobalDoubleDropStrategy);

#[cfg(feature = "global")]
impl<T: crate::ThreadSafe + DoubleDropStrategy> GlobalDoubleDropStrategy for T {}

#[cfg_attr(
    feature = "derives",
    derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)
)]
pub struct DoubleDropStrategyHandler<DDS, FTDS>
where
    DDS: DoubleDropStrategy,
    FTDS: FallibleTryDropStrategy,
{
    pub double_drop_strategy: DDS,
    pub fallible_try_drop_strategy: FTDS,
}

impl<DDS, FTDS> DoubleDropStrategyHandler<DDS, FTDS>
where
    DDS: DoubleDropStrategy,
    FTDS: FallibleTryDropStrategy,
{
    pub fn new(double_drop_strategy: DDS, fallible_try_drop_strategy: FTDS) -> Self {
        Self {
            double_drop_strategy,
            fallible_try_drop_strategy,
        }
    }
}

impl<DDS, FTDS> TryDropStrategy for DoubleDropStrategyHandler<DDS, FTDS>
where
    DDS: DoubleDropStrategy,
    FTDS: FallibleTryDropStrategy,
{
    fn handle_error(&self, error: anyhow::Error) {
        if let Err(error) = self.fallible_try_drop_strategy.dyn_try_handle_error(error) {
            self.double_drop_strategy
                .handle_error_in_drop_strategy(error)
        }
    }
}
