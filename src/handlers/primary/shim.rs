use std::sync::atomic::{AtomicBool, Ordering};
use anyhow::Error;
use crate::handlers::primary::global::GlobalPrimaryDropStrategy;
use crate::handlers::primary::thread_local::ThreadLocalPrimaryTryDropStrategy;
use crate::on_uninit::{DoNothingOnUninit, ErrorOnUninit, FlagOnUninit, PanicOnUninit};
use crate::FallibleTryDropStrategy;
use crate::handlers::shim::OnUninitShim;
#[cfg(feature = "ds-write")]
mod imp {
    use std::io;
    use once_cell::sync::Lazy;
    use crate::drop_strategies::WriteDropStrategy;
    use crate::FallibleTryDropStrategy;
    use crate::handlers::primary::global::GlobalPrimaryDropStrategy;
    use crate::handlers::primary::shim::ShimPrimaryDropStrategy;
    use crate::handlers::primary::thread_local::ThreadLocalPrimaryTryDropStrategy;
    use crate::handlers::shim::{PrimaryHandler, UseDefaultOnUninitShim};

    pub type DefaultOnUninit = UseDefaultOnUninitShim<PrimaryHandler>;

    impl ShimPrimaryDropStrategy<DefaultOnUninit> {
        pub const DEFAULT: Self = Self::USE_DEFAULT_ON_UNINIT;
    }

    impl ShimPrimaryDropStrategy<UseDefaultOnUninitShim<PrimaryHandler>> {
        pub const USE_DEFAULT_ON_UNINIT: Self = Self {
            global: GlobalPrimaryDropStrategy::FLAG_ON_UNINIT,
            thread_local: ThreadLocalPrimaryTryDropStrategy::FLAG_ON_UNINIT,
            extra_data: Lazy::new(|| {
                let mut strategy = WriteDropStrategy::stderr();
                strategy.prelude("error: ");
                strategy
            }),
        };

        pub const fn use_default_on_uninit() -> Self {
            Self::USE_DEFAULT_ON_UNINIT
        }

        fn cache(&self) -> &WriteDropStrategy<io::Stderr> {
            &self.extra_data
        }
    }

    impl FallibleTryDropStrategy for ShimPrimaryDropStrategy<UseDefaultOnUninitShim<PrimaryHandler>> {
        type Error = anyhow::Error;

        fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
            self.on_all_uninit(error, |_, error| self.cache().try_handle_error(error.into()).map_err(Into::into))
        }
    }
}

#[cfg(not(feature = "ds-write"))]
mod imp {
    use crate::handlers::primary::shim::ShimPrimaryDropStrategy;
    use crate::on_uninit::PanicOnUninit;

    pub type DefaultOnUninit = PanicOnUninit;

    impl ShimPrimaryDropStrategy<DefaultOnUninit> {
        pub const DEFAULT: Self = Self::PANIC_ON_UNINIT;
    }
}

pub use imp::DefaultOnUninit;
use crate::adapters::ArcError;

pub static DEFAULT_SHIM_PRIMARY_DROP_STRATEGY: ShimPrimaryDropStrategy = ShimPrimaryDropStrategy::DEFAULT;

#[cfg_attr(
    feature = "derives",
    derive(Debug)
)]
pub struct ShimPrimaryDropStrategy<OU: OnUninitShim = DefaultOnUninit> {
    global: GlobalPrimaryDropStrategy<FlagOnUninit>,
    thread_local: ThreadLocalPrimaryTryDropStrategy<FlagOnUninit>,
    extra_data: OU::ExtraData,
}

impl ShimPrimaryDropStrategy<ErrorOnUninit> {
    pub const ERROR_ON_UNINIT: Self = Self {
        global: GlobalPrimaryDropStrategy::FLAG_ON_UNINIT,
        thread_local: ThreadLocalPrimaryTryDropStrategy::FLAG_ON_UNINIT,
        extra_data: (),
    };

    pub const fn on_uninit_error() -> Self {
        Self::ERROR_ON_UNINIT
    }
}

impl ShimPrimaryDropStrategy<PanicOnUninit> {
    pub const PANIC_ON_UNINIT: Self = Self {
        global: GlobalPrimaryDropStrategy::FLAG_ON_UNINIT,
        thread_local: ThreadLocalPrimaryTryDropStrategy::FLAG_ON_UNINIT,
        extra_data: (),
    };

    pub const fn on_uninit_panic() -> Self {
        Self::PANIC_ON_UNINIT
    }
}

impl ShimPrimaryDropStrategy<DoNothingOnUninit> {
    pub const DO_NOTHING_ON_UNINIT: Self = Self {
        global: GlobalPrimaryDropStrategy::FLAG_ON_UNINIT,
        thread_local: ThreadLocalPrimaryTryDropStrategy::FLAG_ON_UNINIT,
        extra_data: (),
    };

    pub const fn on_uninit_do_nothing() -> Self {
        Self::DO_NOTHING_ON_UNINIT
    }
}

impl ShimPrimaryDropStrategy<FlagOnUninit> {
    pub const FLAG_ON_UNINIT: Self = Self {
        global: GlobalPrimaryDropStrategy::FLAG_ON_UNINIT,
        thread_local: ThreadLocalPrimaryTryDropStrategy::FLAG_ON_UNINIT,
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

impl<OU: OnUninitShim> ShimPrimaryDropStrategy<OU> {
    fn on_all_uninit(&self, error: anyhow::Error, f: impl FnOnce(anyhow::Error, ArcError) -> anyhow::Result<()>) -> anyhow::Result<()> {
        let error = ArcError::new(error);

        match self.thread_local.try_handle_error(ArcError::clone(&error).into()) {
            Ok(()) => Ok(()),
            Err(_) if self.thread_local.last_drop_failed() => match self.global.try_handle_error(ArcError::clone(&error).into()) {
                Ok(()) => Ok(()),
                Err(uninit_error) if self.global.last_drop_failed() => f(uninit_error, error),
                Err(error) => Err(error),
            }
            Err(error) => Err(error),
        }
    }
}

impl FallibleTryDropStrategy for ShimPrimaryDropStrategy<ErrorOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        self.on_all_uninit(error, |uninit_error, _| Err(uninit_error))
    }
}

impl FallibleTryDropStrategy for ShimPrimaryDropStrategy<PanicOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        self.on_all_uninit(
            error,
            |_, error| panic!("neither the thread local nor the global drop strategies are initialized (but here's the drop error anyway: {error})")
        )
    }
}

impl FallibleTryDropStrategy for ShimPrimaryDropStrategy<DoNothingOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        self.on_all_uninit(error, |_, _| Ok(()))
    }
}

impl FallibleTryDropStrategy for ShimPrimaryDropStrategy<FlagOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        let mut last_drop_failed = false;
        let result = self.on_all_uninit(error, |uninit_error, _| {
            last_drop_failed = true;
            Err(uninit_error.into())
        });

        self.set_last_drop_failed(last_drop_failed);

        result
    }
}
