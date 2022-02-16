//! Manage the shim fallback handler.

#[cfg(feature = "ds-panic")]
mod imp {
    use super::ShimFallbackHandler;
    use crate::drop_strategies::PanicDropStrategy;
    use crate::handlers::common::handler::CommonHandler;
    use crate::handlers::common::shim::UseDefaultOnUninitShim;
    use crate::handlers::common::Fallback;
    use crate::TryDropStrategy;
    use once_cell::sync::Lazy;

    /// The default thing to do when both the primary and fallback handlers are uninitialized,
    /// that is to use the inner cache to handle the error instead.
    pub type DefaultOnUninit = UseDefaultOnUninitShim<Fallback>;

    impl ShimFallbackHandler<UseDefaultOnUninitShim<Fallback>> {
        /// See [`Self::use_default_on_uninit`].
        pub const USE_DEFAULT_ON_UNINIT: Self = Self {
            global: CommonHandler::FLAG_ON_UNINIT,
            thread_local: CommonHandler::FLAG_ON_UNINIT,
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

    impl TryDropStrategy for ShimFallbackHandler<UseDefaultOnUninitShim<Fallback>> {
        fn handle_error(&self, error: crate::Error) {
            self.on_all_uninit(error, |error| self.cache().handle_error(error.into()))
        }
    }
}
#[cfg(not(feature = "ds-panic"))]
mod imp {
    use super::ShimFallbackHandler;
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
use crate::handlers::common::handler::CommonShimHandler;
use crate::handlers::common::shim::OnUninitShim;
use crate::handlers::common::Fallback;
use crate::handlers::on_uninit::{DoNothingOnUninit, FlagOnUninit, PanicOnUninit};
use crate::TryDropStrategy;
pub use imp::DefaultOnUninit;

/// A fallback handler which uses both the global and thread-local scopes, with the thread-local
/// scope taking precedence.
pub type ShimFallbackHandler<OU = DefaultOnUninit> = CommonShimHandler<OU, Fallback>;

/// The default shim fallback handler.
pub static DEFAULT_SHIM_FALLBACK_HANDLER: ShimFallbackHandler = ShimFallbackHandler::DEFAULT;

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
    fn handle_error(&self, error: crate::Error) {
        self.on_all_uninit(error, |_| ())
    }
}

impl TryDropStrategy for ShimFallbackHandler<FlagOnUninit> {
    fn handle_error(&self, error: crate::Error) {
        let mut last_drop_failed = false;
        self.on_all_uninit(error, |_| last_drop_failed = true);
        self.set_last_drop_failed(last_drop_failed);
    }
}
