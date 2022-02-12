//! Most commonly used traits.

pub use crate::{
    DynFallibleTryDropStrategy, FallibleTryDropStrategy, PureTryDrop, ThreadSafe, TryDrop,
    TryDropStrategy,
};

#[cfg(feature = "global")]
pub use crate::{GlobalDynFallibleTryDropStrategy, GlobalTryDropStrategy};

#[cfg(any(feature = "global", feature = "thread-local"))]
pub use crate::ImpureTryDrop;
