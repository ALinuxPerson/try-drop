//! Manage the primary shim handler.

use crate::handlers::on_uninit::{DoNothingOnUninit, ErrorOnUninit, FlagOnUninit, PanicOnUninit};
use crate::handlers::primary::global::GlobalPrimaryHandler;
use crate::handlers::primary::thread_local::ThreadLocalPrimaryHandler;
use crate::handlers::common::shim::OnUninitShim;
use crate::{FallibleTryDropStrategy, LOAD_ORDERING, STORE_ORDERING};
use anyhow::Error;
use std::sync::atomic::AtomicBool;
#[cfg(feature = "ds-write")]
mod imp {
    use crate::drop_strategies::WriteDropStrategy;
    use crate::handlers::primary::global::GlobalPrimaryHandler;
    use crate::handlers::primary::shim::ShimPrimaryHandler;
    use crate::handlers::primary::thread_local::ThreadLocalPrimaryHandler;
    use crate::handlers::common::Primary;
    use crate::handlers::common::shim::UseDefaultOnUninitShim;
    use crate::FallibleTryDropStrategy;
    use once_cell::sync::Lazy;
    use std::io;

    /// The default thing to do when both the global and thread-local primary handlers are
    /// uninitialized, that is to use the internal cache.
    pub type DefaultOnUninit = UseDefaultOnUninitShim<Primary>;

    impl ShimPrimaryHandler<DefaultOnUninit> {
        /// The default shim primary handler.
        pub const DEFAULT: Self = Self::USE_DEFAULT_ON_UNINIT;
    }

    impl ShimPrimaryHandler<UseDefaultOnUninitShim<Primary>> {
        /// See [`Self::use_default_on_uninit`].
        #[allow(clippy::declare_interior_mutable_const)]
        pub const USE_DEFAULT_ON_UNINIT: Self = Self {
            global: GlobalPrimaryHandler::FLAG_ON_UNINIT,
            thread_local: ThreadLocalPrimaryHandler::FLAG_ON_UNINIT,
            extra_data: Lazy::new(|| {
                let mut strategy = WriteDropStrategy::stderr();
                strategy.prelude("error: ");
                strategy
            }),
        };

        /// When both the global and thread-local primary handlers are uninitialized, use the
        /// internal cache.
        pub const fn use_default_on_uninit() -> Self {
            Self::USE_DEFAULT_ON_UNINIT
        }

        fn cache(&self) -> &WriteDropStrategy<io::Stderr> {
            &self.extra_data
        }
    }

    impl FallibleTryDropStrategy for ShimPrimaryHandler<UseDefaultOnUninitShim<Primary>> {
        type Error = anyhow::Error;

        fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
            self.on_all_uninit(error, |_, error| {
                self.cache()
                    .try_handle_error(error.into())
                    .map_err(Into::into)
            })
        }
    }
}

#[cfg(not(feature = "ds-write"))]
mod imp {
    use crate::handlers::on_uninit::PanicOnUninit;
    use crate::handlers::primary::shim::ShimPrimaryHandler;

    /// The default thing to do when both the global and thread-local primary handlers are
    /// uninitialized, that is to panic.
    pub type DefaultOnUninit = PanicOnUninit;

    impl ShimPrimaryHandler<DefaultOnUninit> {
        pub const DEFAULT: Self = Self::PANIC_ON_UNINIT;
    }
}

use crate::adapters::ArcError;
pub use imp::DefaultOnUninit;

/// The default shim primary handler.
pub static DEFAULT_SHIM_PRIMARY_HANDLER: ShimPrimaryHandler =
    ShimPrimaryHandler::DEFAULT;

/// A primary handler which merges the global and thread-local primary handlers together, with
/// the thread-local primary handler taking precedence.
#[cfg_attr(feature = "derives", derive(Debug))]
pub struct ShimPrimaryHandler<OU: OnUninitShim = DefaultOnUninit> {
    global: GlobalPrimaryHandler<FlagOnUninit>,
    thread_local: ThreadLocalPrimaryHandler<FlagOnUninit>,
    extra_data: OU::ExtraData,
}

impl ShimPrimaryHandler<ErrorOnUninit> {
    /// See [`Self::on_uninit_error`].
    #[allow(clippy::declare_interior_mutable_const)]
    pub const ERROR_ON_UNINIT: Self = Self {
        global: GlobalPrimaryHandler::FLAG_ON_UNINIT,
        thread_local: ThreadLocalPrimaryHandler::FLAG_ON_UNINIT,
        extra_data: (),
    };

    /// If both the global and thread-local primary handlers are uninitialized, then return an
    /// error.
    pub const fn on_uninit_error() -> Self {
        Self::ERROR_ON_UNINIT
    }
}

impl ShimPrimaryHandler<PanicOnUninit> {
    /// See [`Self::on_uninit_panic`].
    #[allow(clippy::declare_interior_mutable_const)]
    pub const PANIC_ON_UNINIT: Self = Self {
        global: GlobalPrimaryHandler::FLAG_ON_UNINIT,
        thread_local: ThreadLocalPrimaryHandler::FLAG_ON_UNINIT,
        extra_data: (),
    };

    /// If both the global and thread-local primary handlers are uninitialized, then panic.
    pub const fn on_uninit_panic() -> Self {
        Self::PANIC_ON_UNINIT
    }
}

impl ShimPrimaryHandler<DoNothingOnUninit> {
    /// See [`Self::on_uninit_do_nothing`].
    #[allow(clippy::declare_interior_mutable_const)]
    pub const DO_NOTHING_ON_UNINIT: Self = Self {
        global: GlobalPrimaryHandler::FLAG_ON_UNINIT,
        thread_local: ThreadLocalPrimaryHandler::FLAG_ON_UNINIT,
        extra_data: (),
    };

    /// If both the global and thread-local primary handlers are uninitialized, then do nothing.
    pub const fn on_uninit_do_nothing() -> Self {
        Self::DO_NOTHING_ON_UNINIT
    }
}

impl ShimPrimaryHandler<FlagOnUninit> {
    /// See [`Self::on_uninit_flag`].
    #[allow(clippy::declare_interior_mutable_const)]
    pub const FLAG_ON_UNINIT: Self = Self {
        global: GlobalPrimaryHandler::FLAG_ON_UNINIT,
        thread_local: ThreadLocalPrimaryHandler::FLAG_ON_UNINIT,
        extra_data: AtomicBool::new(false),
    };

    /// If both the global and thread-local primary handlers are uninitialized, then
    /// `last_drop_failed` will be set to `true`.
    pub const fn on_uninit_flag() -> Self {
        Self::FLAG_ON_UNINIT
    }

    /// If the last attempt to handle a drop error failed due to both the global and thread-local
    /// primary handlers being uninitialized, then this method will return `true`.
    pub fn last_drop_failed(&self) -> bool {
        self.extra_data.load(LOAD_ORDERING)
    }

    fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, STORE_ORDERING)
    }
}

impl<OU: OnUninitShim> ShimPrimaryHandler<OU> {
    fn on_all_uninit(
        &self,
        error: anyhow::Error,
        f: impl FnOnce(anyhow::Error, ArcError) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let error = ArcError::new(error);

        match self
            .thread_local
            .try_handle_error(ArcError::clone(&error).into())
        {
            Ok(()) => Ok(()),
            Err(_) if self.thread_local.last_drop_failed() => {
                match self.global.try_handle_error(ArcError::clone(&error).into()) {
                    Ok(()) => Ok(()),
                    Err(uninit_error) if self.global.last_drop_failed() => f(uninit_error, error),
                    Err(error) => Err(error),
                }
            }
            Err(error) => Err(error),
        }
    }
}

impl FallibleTryDropStrategy for ShimPrimaryHandler<ErrorOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        self.on_all_uninit(error, |uninit_error, _| Err(uninit_error))
    }
}

impl FallibleTryDropStrategy for ShimPrimaryHandler<PanicOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        self.on_all_uninit(
            error,
            |_, error| panic!("neither the thread local nor the global primary handlers are initialized (but here's the drop error anyway: {error})")
        )
    }
}

impl FallibleTryDropStrategy for ShimPrimaryHandler<DoNothingOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        self.on_all_uninit(error, |_, _| Ok(()))
    }
}

impl FallibleTryDropStrategy for ShimPrimaryHandler<FlagOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: Error) -> Result<(), Self::Error> {
        let mut last_drop_failed = false;
        let result = self.on_all_uninit(error, |uninit_error, _| {
            last_drop_failed = true;
            Err(uninit_error)
        });

        self.set_last_drop_failed(last_drop_failed);

        result
    }
}
