use crate::fallback::{FallbackTryDropStrategyHandler, FallbackTryDropStrategyRef};
use crate::{FallibleTryDropStrategyRef, PureTryDrop, TryDropStrategy};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
#[cfg_attr(feature = "shrinkwraprs", derive(Shrinkwrap))]
#[cfg_attr(feature = "shrinkwraprs", shrinkwrap(mutable))]
pub struct DropAdapter<TD: PureTryDrop>(pub TD);

impl<TD: PureTryDrop> Drop for DropAdapter<TD> {
    fn drop(&mut self) {
        // SAFETY: we called this function inside a `Drop::drop` context.
        let result = unsafe { self.0.try_drop() };
        if let Err(error) = result {
            let handler = FallbackTryDropStrategyHandler::new(
                FallbackTryDropStrategyRef(self.0.fallback_try_drop_strategy()),
                FallibleTryDropStrategyRef(self.0.drop_strategy()),
            );

            handler.handle_error(error.into())
        }
    }
}
