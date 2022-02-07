pub use crate::{
    double::DoubleDropStrategy, DynFallibleTryDropStrategy, FallibleTryDropStrategy,
    PureTryDrop, ThreadSafe, TryDrop, TryDropStrategy,
};

#[cfg(feature = "global")]
pub use crate::{double::GlobalDoubleDropStrategy, GlobalDynFallibleTryDropStrategy};
