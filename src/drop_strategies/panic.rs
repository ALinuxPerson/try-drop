use crate::{Error, TryDropStrategy};
use std::borrow::Cow;
use std::string::String;

#[cfg_attr(
    feature = "derives",
    derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)
)]
pub struct PanicDropStrategy {
    pub message: Cow<'static, str>,
}

impl PanicDropStrategy {
    pub const fn new() -> Self {
        Self::with_static_message("error occured when dropping an object")
    }

    pub fn with_message(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub const fn with_static_message(message: &'static str) -> Self {
        Self {
            message: Cow::Borrowed(message),
        }
    }

    pub const fn with_dynamic_message(message: String) -> Self {
        Self {
            message: Cow::Owned(message),
        }
    }
}

impl TryDropStrategy for PanicDropStrategy {
    fn handle_error(&self, error: Error) {
        Err(error).expect(&self.message)
    }
}

impl Default for PanicDropStrategy {
    fn default() -> Self {
        Self::new()
    }
}
