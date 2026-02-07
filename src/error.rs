use std::{ffi::NulError, str::Utf8Error};

use espeak_sys::{
    espeak_ERROR, espeak_ERROR_EE_BUFFER_FULL, espeak_ERROR_EE_INTERNAL_ERROR,
    espeak_ERROR_EE_NOT_FOUND,
};

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("espeak operation failed: {}", espeak_error_msg(*.0))]
    Espeak(espeak_ERROR),

    #[error("no voices available")]
    NoVoicesAvailable,

    #[error(transparent)]
    NullPointer(#[from] NulError),

    #[error(transparent)]
    InvalidUtf8(#[from] Utf8Error),
}

fn espeak_error_msg(code: espeak_ERROR) -> &'static str {
    match code {
        espeak_ERROR_EE_INTERNAL_ERROR => "internal error",
        espeak_ERROR_EE_BUFFER_FULL => "buffer full",
        espeak_ERROR_EE_NOT_FOUND => "not found",
        _ => "unknown error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn espeak_variant_to_string_returns_expected() {
        assert_eq!(
            Error::Espeak(espeak_ERROR_EE_INTERNAL_ERROR).to_string(),
            "espeak operation failed: internal error"
        );
        assert_eq!(
            Error::Espeak(espeak_ERROR_EE_BUFFER_FULL).to_string(),
            "espeak operation failed: buffer full"
        );
        assert_eq!(
            Error::Espeak(espeak_ERROR_EE_NOT_FOUND).to_string(),
            "espeak operation failed: not found"
        );
        assert_eq!(
            Error::Espeak(10).to_string(),
            "espeak operation failed: unknown error"
        );
    }
}
