//! Manage the shim fallback handler.

#![allow(clippy::declare_interior_mutable_const)]
use crate::handlers::fallback::global::GlobalFallbackHandler;
use crate::handlers::fallback::thread_local::ThreadLocalFallbackHandler;
use crate::handlers::on_uninit::{DoNothingOnUninit, FlagOnUninit, PanicOnUninit};
use crate::handlers::shim::OnUninitShim;
use crate::{TryDropStrategy, LOAD_ORDERING, STORE_ORDERING};
use anyhow::Error;
use std::sync::atomic::AtomicBool;
#[cfg(feature = "ds-panic")]
mod imp {
    use crate::drop_strategies::PanicDropStrategy;
    use crate::handlers::fallback::global::GlobalFallbackHandler;
    use crate::handlers::fallback::shim::ShimFallbackHandler;
    use crate::handlers::fallback::thread_local::ThreadLocalFallbackHandler;
    use crate::handlers::shim::{FallbackHandler, UseDefaultOnUninitShim};
    use crate::TryDropStrategy;
    use once_cell::sync::Lazy;

    /// The default thing to do when both the primary and fallback handlers are uninitialized,
    /// that is to use the inner cache to handle the error instead.
    pub type DefaultOnUninit = UseDefaultOnUninitShim<FallbackHandler>;

    impl ShimFallbackHandler<UseDefaultOnUninitShim<FallbackHandler>> {
        /// See [`Self::use_default_on_uninit`].
        pub const USE_DEFAULT_ON_UNINIT: Self = Self {
            global: GlobalFallbackHandler::FLAG_ON_UNINIT,
            thread_local: ThreadLocalFallbackHandler::FLAG_ON_UNINIT,
            extra_data: Lazy::new(|| PanicDropStrategy::DEFAULT),
        };

        /// If both the primary and fallback handlers are uninitialized, use the inner cache to
        /// handle the error instead.
        pub const fn use_default_on_uninit() -> Self {
            Self::USE_DEFAULT_ON_UNINIT
        }

        fn cache(&self) -> &PanicDropStrategy {
            &self.extra_data
        }
    }

    impl ShimFallbackHandler<DefaultOnUninit> {
        /// The default thing to do when both the primary and fallback handlers are uninitialized.
        pub const DEFAULT: Self = Self::USE_DEFAULT_ON_UNINIT;
    }

    impl TryDropStrategy for ShimFallbackHandler<UseDefaultOnUninitShim<FallbackHandler>> {
        fn handle_error(&self, error: crate::Error) {
            self.on_all_uninit(error, |error| self.cache().handle_error(error.into()))
        }
    }
}
#[cfg(not(feature = "ds-panic"))]
mod imp {
    use crate::handlers::fallback::shim::ShimFallbackHandler;
    use crate::handlers::on_uninit::PanicOnUninit;

    /// The default thing to do when the primary and fallback handlers are uninitialized, that is to
    /// panic.
    pub type DefaultOnUninit = PanicOnUninit;

    impl ShimFallbackHandler<DefaultOnUninit> {
        /// The default thing to do when the primary and fallback handlers are uninitialized.
        pub const DEFAULT: Self = Self::PANIC_ON_UNINIT;
    }
}

use crate::adapters::ArcError;
pub use imp::DefaultOnUninit;

/// The default shim fallback handler.
pub static DEFAULT_SHIM_FALLBACK_HANDLER: ShimFallbackHandler =
    ShimFallbackHandler::DEFAULT;

/// A shim which abstracts the global and thread local handlers together, with the thread local
/// handlers taking precedence over the global handlers.
#[cfg_attr(feature = "derives", derive(Debug))]
pub struct ShimFallbackHandler<OU: OnUninitShim = DefaultOnUninit> {
    global: GlobalFallbackHandler<FlagOnUninit>,
    thread_local: ThreadLocalFallbackHandler<FlagOnUninit>,
    extra_data: OU::ExtraData,
}

impl ShimFallbackHandler<PanicOnUninit> {
    /// See [`Self::on_uninit_panic`].
    pub const PANIC_ON_UNINIT: Self = Self {
        global: GlobalFallbackHandler::FLAG_ON_UNINIT,
        thread_local: ThreadLocalFallbackHandler::FLAG_ON_UNINIT,
        extra_data: (),
    };

    /// When both the primary and fallback handlers are not initialized, panic.
    pub const fn on_uninit_panic() -> Self {
        Self::PANIC_ON_UNINIT
    }
}

impl ShimFallbackHandler<DoNothingOnUninit> {
    /// See [`Self::on_uninit_do_nothing`].
    pub const DO_NOTHING_ON_UNINIT: Self = Self {
        global: GlobalFallbackHandler::FLAG_ON_UNINIT,
        thread_local: ThreadLocalFallbackHandler::FLAG_ON_UNINIT,
        extra_data: (),
    };

    /// When both the primary and fallback handlers are not initialized, do nothing.
    pub const fn on_uninit_do_nothing() -> Self {
        Self::DO_NOTHING_ON_UNINIT
    }
}

impl ShimFallbackHandler<FlagOnUninit> {
    /// See [`Self::on_uninit_flag`].
    pub const FLAG_ON_UNINIT: Self = Self {
        global: GlobalFallbackHandler::FLAG_ON_UNINIT,
        thread_local: ThreadLocalFallbackHandler::FLAG_ON_UNINIT,
        extra_data: AtomicBool::new(false),
    };

    /// When both the primary and fallback handlers are not initialized, set `last_drop_failed` to
    /// `true`.
    pub const fn on_uninit_flag() -> Self {
        Self::FLAG_ON_UNINIT
    }

    /// Check whether or not the last drop failed due to the primary and fallback drop strategies
    /// not being initialized.
    pub fn last_drop_failed(&self) -> bool {
        self.extra_data.load(LOAD_ORDERING)
    }

    fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, STORE_ORDERING)
    }
}

impl<OU: OnUninitShim> ShimFallbackHandler<OU> {
    fn on_all_uninit(&self, error: anyhow::Error, f: impl FnOnce(ArcError)) {
        let error = ArcError::new(error);
        self.thread_local
            .handle_error(ArcError::clone(&error).into());

        if self.thread_local.last_drop_failed() {
            self.global.handle_error(ArcError::clone(&error).into());

            if self.global.last_drop_failed() {
                f(error)
            }
        }
    }
}

impl TryDropStrategy for ShimFallbackHandler<PanicOnUninit> {
    fn handle_error(&self, error: crate::Error) {
        self.on_all_uninit(
            error,
            |error| panic!("neither the fallback thread local nor the fallback global handlers are initialized (but here's the drop error anyway: {error})")
        )
    }
}

impl TryDropStrategy for ShimFallbackHandler<DoNothingOnUninit> {
    fn handle_error(&self, error: Error) {
        self.on_all_uninit(error, |_| ())
    }
}

impl TryDropStrategy for ShimFallbackHandler<FlagOnUninit> {
    fn handle_error(&self, error: Error) {
        let mut last_drop_failed = false;
        self.on_all_uninit(error, |_| last_drop_failed = true);
        self.set_last_drop_failed(last_drop_failed);
    }
}

impl Default for ShimFallbackHandler<DefaultOnUninit> {
    fn default() -> Self {
        Self::DEFAULT
    }
}
