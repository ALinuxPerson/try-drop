use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use crate::handlers::common::{Global, Handler, Scope, ThreadLocal};
use crate::handlers::on_uninit::{DoNothingOnUninit, FlagOnUninit, OnUninit, PanicOnUninit};
use crate::{LOAD_ORDERING, STORE_ORDERING};
use crate::handlers::common::shim::OnUninitShim;

pub struct CommonHandler<OU: OnUninit, S: Scope, H: Handler> {
    pub(crate) extra_data: OU::ExtraData,
    pub(crate) _scope: PhantomData<(S, H)>,
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

pub struct CommonShimHandler<OU: OnUninitShim, H: Handler> {
    pub(crate) global: CommonHandler<FlagOnUninit, Global, H>,
    pub(crate) thread_local: CommonHandler<FlagOnUninit, ThreadLocal, H>,
    pub(crate) extra_data: OU::ExtraData,
}

impl<H: Handler> CommonShimHandler<PanicOnUninit, H> {
    pub const PANIC_ON_UNINIT: Self = Self {
        global: CommonHandler::FLAG_ON_UNINIT,
        thread_local: CommonHandler::FLAG_ON_UNINIT,
        extra_data: (),
    };

    pub fn on_uninit_panic() -> Self {
        Self::PANIC_ON_UNINIT
    }
}

impl<H: Handler> CommonShimHandler<DoNothingOnUninit, H> {
    pub const DO_NOTHING_ON_UNINIT: Self = Self {
        global: CommonHandler::FLAG_ON_UNINIT,
        thread_local: CommonHandler::FLAG_ON_UNINIT,
        extra_data: (),
    };

    pub fn on_uninit_do_nothing() -> Self {
        Self::DO_NOTHING_ON_UNINIT
    }
}

impl<H: Handler> CommonShimHandler<FlagOnUninit, H> {
    pub const FLAG_ON_UNINIT: Self = Self {
        global: CommonHandler::FLAG_ON_UNINIT,
        thread_local: CommonHandler::FLAG_ON_UNINIT,
        extra_data: AtomicBool::new(false),
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