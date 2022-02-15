macro_rules! impl_try_drop_strategy_for {
    ($handler:ident where Scope: $scope:ident) => {
        impl TryDropStrategy for $handler<PanicOnUninit> {
            fn handle_error(&self, error: crate::Error) {
                Abstracter::<$scope>::read(|strategy| strategy.handle_error(error))
            }
        }

        #[cfg(feature = "ds-write")]
        impl TryDropStrategy for $handler<UseDefaultOnUninit> {
            fn handle_error(&self, error: Error) {
                Abstracter::<$scope>::read_or_default(|strategy| strategy.handle_error(error))
            }
        }

        impl TryDropStrategy for $handler<FlagOnUninit> {
            fn handle_error(&self, error: Error) {
                if let Err(UninitializedError(())) =
                    Abstracter::<$scope>::try_read(|strategy| strategy.handle_error(error))
                {
                    self.set_last_drop_failed(true)
                } else {
                    self.set_last_drop_failed(false)
                }
            }
        }
    };
}
