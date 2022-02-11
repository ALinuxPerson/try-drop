//! Most commonly used traits.

pub use crate::{
    DynFallibleTryDropStrategy, FallibleTryDropStrategy,
    PureTryDrop, ThreadSafe, TryDrop, TryDropStrategy,
};

#[cfg(feature = "global")]
pub use crate::{
    GlobalTryDropStrategy, GlobalDynFallibleTryDropStrategy, ImpureTryDrop,
};
