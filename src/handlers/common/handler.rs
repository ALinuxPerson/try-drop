use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use crate::handlers::common::{Handler, Scope};
use crate::handlers::on_uninit::{FlagOnUninit, OnUninit, PanicOnUninit};
use crate::{LOAD_ORDERING, STORE_ORDERING};

pub struct CommonHandler<OU: OnUninit, S: Scope, H: Handler> {
    extra_data: OU::ExtraData,
    _scope: PhantomData<(S, H)>,
}

impl<S: Scope, H: Handler> CommonHandler<PanicOnUninit, S, H> {
    pub const PANIC_ON_UNINIT: Self = Self {
        extra_data: (),
        _scope: PhantomData,
    };

    pub fn on_uninit_panic() -> Self {
        Self::PANIC_ON_UNINIT
    }
}

impl<S: Scope, H: Handler> CommonHandler<FlagOnUninit, S, H> {
    pub const FLAG_ON_UNINIT: Self = Self {
        extra_data: AtomicBool::new(false),
        _scope: PhantomData,
    };

    pub fn on_uninit_flag() -> Self {
        Self::FLAG_ON_UNINIT
    }

    pub fn last_drop_failed(&self) -> bool {
        self.extra_data.load(LOAD_ORDERING)
    }

    pub(crate) fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, STORE_ORDERING)
    }
}