//! Numerous strategies for handling drop errors.

#[cfg(feature = "ds-abort")]
mod abort;

#[cfg(feature = "ds-broadcast")]
pub mod broadcast;

#[cfg(feature = "ds-exit")]
mod exit;

#[cfg(feature = "ds-noop")]
mod noop;

#[cfg(feature = "ds-panic")]
mod panic;

#[cfg(feature = "ds-write")]
mod write;

#[cfg(feature = "ds-adhoc")]
mod adhoc;

#[cfg(feature = "ds-once-cell")]
pub mod once_cell;

#[cfg(feature = "ds-unreachable")]
pub mod unreachable;

#[cfg(feature = "ds-abort")]
pub use abort::AbortDropStrategy;

#[cfg(feature = "ds-broadcast")]
pub use broadcast::BroadcastDropStrategy;

#[cfg(feature = "ds-exit")]
pub use exit::ExitDropStrategy;

#[cfg(feature = "ds-noop")]
pub use noop::NoOpDropStrategy;

#[cfg(feature = "ds-panic")]
pub use panic::PanicDropStrategy;

#[cfg(feature = "ds-write")]
pub use write::{ThreadUnsafeWriteDropStrategy, WriteDropStrategy};

#[cfg(feature = "ds-adhoc")]
pub use adhoc::{
    AdHocDropStrategy, AdHocFallibleDropStrategy, IntoAdHocDropStrategy,
    IntoAdHocFallibleDropStrategy,
};

#[cfg(feature = "ds-adhoc-mut")]
pub use adhoc::{
    AdHocMutDropStrategy, AdHocMutFallibleDropStrategy, IntoAdHocMutDropStrategy,
    IntoAdHocMutFallibleDropStrategy, IntoThreadUnsafeAdHocMutDropStrategy,
    IntoThreadUnsafeAdHocMutFallibleDropStrategy, ThreadUnsafeAdHocMutDropStrategy,
    ThreadUnsafeAdHocMutFallibleDropStrategy,
};

#[cfg(feature = "ds-once-cell")]
pub use self::once_cell::{OnceCell, OnceCellDropStrategy, ThreadUnsafeOnceCellDropStrategy};
