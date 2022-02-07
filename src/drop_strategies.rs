#[cfg(feature = "ds-abort")]
pub mod abort {
    use std::process;

    #[cfg_attr(feature = "derives", derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default))]
    pub struct AbortDropStrategy;

    impl TryDropStrategy for AbortDropStrategy {
        fn handle_error(&self, _error: crate::Error) {
            process::abort()
        }
    }
}

#[cfg(feature = "ds-broadcast")]
pub mod broadcast {}

#[cfg(feature = "ds-exit")]
pub mod exit {}

#[cfg(feature = "ds-noop")]
pub mod noop {}

#[cfg(feature = "ds-panic")]
pub mod panic {}

#[cfg(feature = "ds-write")]
pub mod write {}
