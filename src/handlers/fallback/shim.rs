use std::sync::atomic::{AtomicBool, Ordering};
use anyhow::Error;
use crate::on_uninit::{DoNothingOnUninit, FlagOnUninit, PanicOnUninit};
use crate::TryDropStrategy;
use crate::handlers::fallback::global::GlobalFallbackDropStrategy;
use crate::handlers::fallback::thread_local::ThreadLocalFallbackDropStrategy;
use crate::handlers::shim::OnUninitShim;
#[cfg(feature = "ds-panic")]
mod imp {
    use once_cell::sync::Lazy;
    use crate::drop_strategies::PanicDropStrategy;
    use crate::handlers::fallback::global::GlobalFallbackDropStrategy;
    use crate::handlers::fallback::shim::ShimFallbackDropStrategy;
    use crate::handlers::fallback::thread_local::ThreadLocalFallbackDropStrategy;
    use crate::handlers::shim::{FallbackHandler, UseDefaultOnUninitShim};
    use crate::TryDropStrategy;

    pub type DefaultOnUninit = UseDefaultOnUninitShim<FallbackHandler>;

    impl ShimFallbackDropStrategy<UseDefaultOnUninitShim<FallbackHandler>> {
        pub const USE_DEFAULT_ON_UNINIT: Self = Self {
            global: GlobalFallbackDropStrategy::FLAG_ON_UNINIT,
            thread_local: ThreadLocalFallbackDropStrategy::FLAG_ON_UNINIT,
            extra_data: Lazy::new(|| PanicDropStrategy::DEFAULT),
        };

        pub const fn use_default_on_uninit() -> Self {
            Self::USE_DEFAULT_ON_UNINIT
        }

        fn cache(&self) -> &PanicDropStrategy {
            &self.extra_data
        }
    }

    impl ShimFallbackDropStrategy<DefaultOnUninit> {
        pub const DEFAULT: Self = Self::USE_DEFAULT_ON_UNINIT;
    }

    impl TryDropStrategy for ShimFallbackDropStrategy<UseDefaultOnUninitShim<FallbackHandler>> {
        fn handle_error(&self, error: crate::Error) {
            self.on_all_uninit(error, |error| self.cache().handle_error(error.into()))
        }
    }
}
#[cfg(not(feature = "ds-panic"))]
mod imp {
    use crate::handlers::fallback::shim::ShimFallbackDropStrategy;
    use crate::on_uninit::PanicOnUninit;

    pub type DefaultOnUninit = PanicOnUninit;

    impl ShimFallbackDropStrategy<DefaultOnUninit> {
        pub const DEFAULT: Self = Self::PANIC_ON_UNINIT;
    }
}

pub use imp::DefaultOnUninit;
use crate::adapters::ArcError;

pub static DEFAULT_SHIM_FALLBACK_DROP_STRATEGY: ShimFallbackDropStrategy = ShimFallbackDropStrategy::DEFAULT;

pub struct ShimFallbackDropStrategy<OU: OnUninitShim = DefaultOnUninit> {
    global: GlobalFallbackDropStrategy<FlagOnUninit>,
    thread_local: ThreadLocalFallbackDropStrategy<FlagOnUninit>,
    extra_data: OU::ExtraData,
}

impl ShimFallbackDropStrategy<PanicOnUninit> {
    pub const PANIC_ON_UNINIT: Self = Self {
        global: GlobalFallbackDropStrategy::FLAG_ON_UNINIT,
        thread_local: ThreadLocalFallbackDropStrategy::FLAG_ON_UNINIT,
        extra_data: (),
    };

    pub const fn on_uninit_panic() -> Self {
        Self::PANIC_ON_UNINIT
    }
}

impl ShimFallbackDropStrategy<DoNothingOnUninit> {
    pub const DO_NOTHING_ON_UNINIT: Self = Self {
        global: GlobalFallbackDropStrategy::FLAG_ON_UNINIT,
        thread_local: ThreadLocalFallbackDropStrategy::FLAG_ON_UNINIT,
        extra_data: (),
    };

    pub const fn on_uninit_do_nothing() -> Self {
        Self::DO_NOTHING_ON_UNINIT
    }
}

impl ShimFallbackDropStrategy<FlagOnUninit> {
    pub const FLAG_ON_UNINIT: Self = Self {
        global: GlobalFallbackDropStrategy::FLAG_ON_UNINIT,
        thread_local: ThreadLocalFallbackDropStrategy::FLAG_ON_UNINIT,
        extra_data: AtomicBool::new(false),
    };

    pub const fn flag_on_uninit() -> Self {
        Self::FLAG_ON_UNINIT
    }

    pub fn last_drop_failed(&self) -> bool {
        self.extra_data.load(Ordering::Acquire)
    }

    fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, Ordering::Release)
    }
}

impl<OU: OnUninitShim> ShimFallbackDropStrategy<OU> {
    fn on_all_uninit(&self, error: anyhow::Error, f: impl FnOnce(ArcError)) {
        let error = ArcError::new(error);
        self.thread_local.handle_error(ArcError::clone(&error).into());

        if self.thread_local.last_drop_failed() {
            self.global.handle_error(ArcError::clone(&error).into());

            if self.global.last_drop_failed() {
                f(error)
            }
        }
    }
}

impl TryDropStrategy for ShimFallbackDropStrategy<PanicOnUninit> {
    fn handle_error(&self, error: crate::Error) {
        self.on_all_uninit(
            error,
            |error| panic!("neither the fallback thread local nor the fallback global drop strategies are initialized (but here's the drop error anyway: {error})")
        )
    }
}

impl TryDropStrategy for ShimFallbackDropStrategy<DoNothingOnUninit> {
    fn handle_error(&self, error: Error) {
        self.on_all_uninit(error, |_| ())
    }
}

impl TryDropStrategy for ShimFallbackDropStrategy<FlagOnUninit> {
    fn handle_error(&self, error: Error) {
        let mut last_drop_failed = false;
        self.on_all_uninit(error, |_| last_drop_failed = true);
        self.set_last_drop_failed(last_drop_failed);
    }
}

