macro_rules! impl_fallible_try_drop_strategy_for {
    ($handler:ident where Scope: $scope:ident, Definition: $definition:ident) => {
        impl FallibleTryDropStrategy for $handler<ErrorOnUninit> {
            type Error = anyhow::Error;

            fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
                Abstracter::<$scope>::try_read(|strategy| strategy.dyn_try_handle_error(error))
                    .map_err(Into::into)
                    .and_then(convert::identity)
            }
        }

        impl FallibleTryDropStrategy for $handler<PanicOnUninit> {
            type Error = anyhow::Error;

            fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
                Abstracter::<$scope>::try_read(|strategy| strategy.dyn_try_handle_error(error))
                    .expect(<Primary as $definition>::UNINITIALIZED_ERROR)
            }
        }

        #[cfg(feature = "ds-write")]
        impl FallibleTryDropStrategy for $handler<UseDefaultOnUninit> {
            type Error = anyhow::Error;

            fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
                Abstracter::<$scope>::read_or_default(|strategy| {
                    strategy.dyn_try_handle_error(error)
                })
            }
        }

        impl FallibleTryDropStrategy for $handler<FlagOnUninit> {
            type Error = anyhow::Error;

            fn try_handle_error(&self, error: crate::Error) -> Result<(), Self::Error> {
                let (last_drop_failed, ret) =
                    match Abstracter::<$scope>::try_read(|s| s.dyn_try_handle_error(error)) {
                        Ok(Ok(())) => (false, Ok(())),
                        Ok(Err(error)) => (false, Err(error)),
                        Err(error) => (true, Err(error.into())),
                    };
                self.set_last_drop_failed(last_drop_failed);
                ret
            }
        }
    };
}
