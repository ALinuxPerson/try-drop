use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use crate::handlers::common::Scope;
use crate::handlers::on_uninit::{FlagOnUninit, OnUninit, PanicOnUninit};
use crate::{LOAD_ORDERING, STORE_ORDERING};

pub struct Handler<OU: OnUninit, S: Scope> {
    extra_data: OU::ExtraData,
    _scope: PhantomData<S>,
}

impl<S: Scope> Handler<PanicOnUninit, S> {
    pub const PANIC_ON_UNINIT: Self = Self {
        extra_data: (),
        _scope: PhantomData,
    };

    pub fn on_uninit_panic() -> Self {
        Self::PANIC_ON_UNINIT
    }
}

impl<S: Scope> Handler<FlagOnUninit, S> {
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

    fn set_last_drop_failed(&self, value: bool) {
        self.extra_data.store(value, STORE_ORDERING)
    }
}