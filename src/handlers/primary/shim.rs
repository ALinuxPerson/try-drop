//! Manage the primary shim handler.

#[cfg(feature = "ds-write")]
mod imp {
    use super::ShimPrimaryHandler;
    use crate::drop_strategies::WriteDropStrategy;
    use crate::handlers::common::handler::CommonHandler;
    use crate::handlers::common::shim::UseDefaultOnUninitShim;
    use crate::handlers::common::Primary;
    
    
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
            global: CommonHandler::FLAG_ON_UNINIT,
            thread_local: CommonHandler::FLAG_ON_UNINIT,
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
    use super::ShimPrimaryHandler;
    use crate::handlers::on_uninit::PanicOnUninit;

    /// The default thing to do when both the global and thread-local primary handlers are
    /// uninitialized, that is to panic.
    pub type DefaultOnUninit = PanicOnUninit;

    impl ShimPrimaryHandler<DefaultOnUninit> {
        pub const DEFAULT: Self = Self::PANIC_ON_UNINIT;
    }
}

use crate::adapters::ArcError;
use crate::handlers::common::handler::CommonShimHandler;
use crate::handlers::common::shim::OnUninitShim;
use crate::handlers::common::Primary;
use crate::handlers::on_uninit::{DoNothingOnUninit, ErrorOnUninit, FlagOnUninit, PanicOnUninit};
use crate::FallibleTryDropStrategy;
pub use imp::DefaultOnUninit;

/// A primary handler whose scope combines both the global and thread-local primary handlers, with
/// the thread-local primary handler taking precedence.
pub type ShimPrimaryHandler<OU = DefaultOnUninit> = CommonShimHandler<OU, Primary>;

/// The default primary handler using the shim scope.
pub static DEFAULT_SHIM_PRIMARY_HANDLER: ShimPrimaryHandler = ShimPrimaryHandler::DEFAULT;

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

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        self.on_all_uninit(error, |_, _| Ok(()))
    }
}

impl FallibleTryDropStrategy for ShimPrimaryHandler<FlagOnUninit> {
    type Error = anyhow::Error;

    fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
        let mut last_drop_failed = false;
        let result = self.on_all_uninit(error, |uninit_error, _| {
            last_drop_failed = true;
            Err(uninit_error)
        });

        self.set_last_drop_failed(last_drop_failed);

        result
    }
}
