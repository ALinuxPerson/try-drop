//! Manage the primary shim handler.

use crate::handlers::on_uninit::{DoNothingOnUninit, ErrorOnUninit, FlagOnUninit, PanicOnUninit};
use crate::handlers::primary::global::GlobalPrimaryHandler;
use crate::handlers::primary::thread_local::ThreadLocalPrimaryHandler;
use crate::handlers::shim::OnUninitShim;
use crate::{FallibleTryDropStrategy, LOAD_ORDERING, STORE_ORDERING};
use anyhow::Error;
use std::sync::atomic::AtomicBool;
#[cfg(feature = "ds-write")]
mod imp {
    use crate::drop_strategies::WriteDropStrategy;
    use crate::handlers::primary::global::GlobalPrimaryHandler;
    use crate::handlers::primary::shim::ShimPrimaryDropStrategy;
    use crate::handlers::primary::thread_local::ThreadLocalPrimaryHandler;
    use crate::handlers::shim::{PrimaryHandler, UseDefaultOnUninitShim};
    use crate::FallibleTryDropStrategy;
    use once_cell::sync::Lazy;
    use std::io;

    /// The default thing to do when both the global and thread-local drop strategies are
    /// uninitialized, that is to use the internal cache.
    pub type DefaultOnUninit = UseDefaultOnUninitShim<PrimaryHandler>;

    impl ShimPrimaryDropStrategy<DefaultOnUninit> {
        /// The default shim primary drop strategy.
        #[allow(clippy::declare_interior_mutable_const)]
        pub const DEFAULT: Self = Self::USE_DEFAULT_ON_UNINIT;
    }

    impl ShimPrimaryDropStrategy<UseDefaultOnUninitShim<PrimaryHandler>> {
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

        /// When both the global and thread-local drop strategies are uninitialized, use the
        /// internal cache.
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
    use crate::handlers::primary::shim::ShimPrimaryDropStrategy;

    /// The default thing to do when both the global and thread-local drop strategies are
    /// uninitialized, that is to panic.
    pub type DefaultOnUninit = PanicOnUninit;

    impl ShimPrimaryDropStrategy<DefaultOnUninit> {
        pub const DEFAULT: Self = Self::PANIC_ON_UNINIT;
    }
}

use crate::adapters::ArcError;
pub use imp::DefaultOnUninit;

/// The default shim primary drop strategy.
pub static DEFAULT_SHIM_PRIMARY_DROP_STRATEGY: ShimPrimaryDropStrategy =
    ShimPrimaryDropStrategy::DEFAULT;

/// A primary drop strategy which merges the global and thread-local drop strategies together, with
/// the thread-local drop strategy taking precedence.
#[cfg_attr(feature = "derives", derive(Debug))]
pub struct ShimPrimaryDropStrategy<OU: OnUninitShim = DefaultOnUninit> {
    global: GlobalPrimaryHandler<FlagOnUninit>,
    thread_local: ThreadLocalPrimaryHandler<FlagOnUninit>,
    extra_data: OU::ExtraData,
}

impl ShimPrimaryDropStrategy<ErrorOnUninit> {
    /// See [`Self::on_uninit_error`].
    #[allow(clippy::declare_interior_mutable_const)]
    pub const ERROR_ON_UNINIT: Self = Self {
        global: GlobalPrimaryHandler::FLAG_ON_UNINIT,
        thread_local: ThreadLocalPrimaryHandler::FLAG_ON_UNINIT,
        extra_data: (),
    };

    /// If both the global and thread-local primary drop strategies are uninitialized, then return
    /// an error.
    pub const fn on_uninit_error() -> Self {
        Self::ERROR_ON_UNINIT
    }
}

impl ShimPrimaryDropStrategy<PanicOnUninit> {
    /// See [`Self::on_uninit_panic`].
    #[allow(clippy::declare_interior_mutable_const)]
    pub const PANIC_ON_UNINIT: Self = Self {
        global: GlobalPrimaryHandler::FLAG_ON_UNINIT,
        thread_local: ThreadLocalPrimaryHandler::FLAG_ON_UNINIT,
        extra_data: (),
    };

    /// If both the global and thread-local primary drop strategies are uninitialized, then panic.
    pub const fn on_uninit_panic() -> Self {
        Self::PANIC_ON_UNINIT
    }
}

impl ShimPrimaryDropStrategy<DoNothingOnUninit> {
    /// See [`Self::on_uninit_do_nothing`].
    #[allow(clippy::declare_interior_mutable_const)]
    pub const DO_NOTHING_ON_UNINIT: Self = Self {
        global: GlobalPrimaryHandler::FLAG_ON_UNINIT,
        thread_local: ThreadLocalPrimaryHandler::FLAG_ON_UNINIT,
        extra_data: (),
    };

    /// If both the global and thread-local primary drop strategies are uninitialized, then do
    /// nothing.
    pub const fn on_uninit_do_nothing() -> Self {
        Self::DO_NOTHING_ON_UNINIT
    }
}

impl ShimPrimaryDropStrategy<FlagOnUninit> {
    /// See [`Self::on_uninit_flag`].
    #[allow(clippy::declare_interior_mutable_const)]
    pub const FLAG_ON_UNINIT: Self = Self {
        global: GlobalPrimaryHandler::FLAG_ON_UNINIT,
        thread_local: ThreadLocalPrimaryHandler::FLAG_ON_UNINIT,
        extra_data: AtomicBool::new(false),
    };

    /// If both the global and thread-local primary drop strategies are uninitialized, then
    /// `last_drop_failed` will be set to `true`.
    pub const fn on_uninit_flag() -> Self {
        Self::FLAG_ON_UNINIT
    }

    /// If the last attempt to handle a drop error failed due to both the global and thread-local
    /// primary drop strategies being uninitialized, then this method will return `true`.
    pub fn last_drop_failed(&self) -> bool {
        self.extra_data.load(LOAD_ORDERING)
    }

    fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, STORE_ORDERING)
    }
}

impl<OU: OnUninitShim> ShimPrimaryDropStrategy<OU> {
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
            Err(uninit_error)
        });

        self.set_last_drop_failed(last_drop_failed);

        result
    }
}
