pub use crate::{
    double::DoubleDropStrategy, DynFallibleTryDropStrategy, FallibleTryDropStrategy,
    SpecificTryDrop, ThreadSafe, TryDrop, TryDropStrategy,
};

#[cfg(feature = "global")]
pub use crate::{double::GlobalDoubleDropStrategy, GlobalDynFallibleTryDropStrategy};
