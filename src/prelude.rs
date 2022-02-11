//! Most commonly used traits.

pub use crate::{
    DynFallibleTryDropStrategy, FallibleTryDropStrategy,
    PureTryDrop, ThreadSafe, TryDrop, TryDropStrategy,
};

#[cfg(any(feature = "global", feature = "thread-local"))]
pub use crate::{
    GlobalTryDropStrategy, GlobalDynFallibleTryDropStrategy, ImpureTryDrop,
};
