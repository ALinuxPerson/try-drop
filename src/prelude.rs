//! Most commonly used traits.

pub use crate::{
    fallback::FallbackTryDropStrategy, DynFallibleTryDropStrategy, FallibleTryDropStrategy,
    PureTryDrop, ThreadSafe, TryDropStrategy, TryDrop,
};

#[cfg(feature = "global")]
pub use crate::{fallback::GlobalFallbackTryDropStrategy, GlobalDynFallibleTryDropStrategy, ImpureTryDrop};
