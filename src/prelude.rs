pub use crate::{
    fallback::FallbackDropStrategy, DynFallibleTryDropStrategy, FallibleTryDropStrategy,
    PureTryDrop, ThreadSafe, ImpureTryDrop, TryDropStrategy,
};

#[cfg(feature = "global")]
pub use crate::{fallback::GlobalFallbackDropStrategy, GlobalDynFallibleTryDropStrategy};
