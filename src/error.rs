#![allow(non_snake_case)]

use std::{ffi::NulError, str::Utf8Error};

use espeak_sys::{
    espeak_ERROR, espeak_ERROR_EE_BUFFER_FULL as BUFFER_FULL,
    espeak_ERROR_EE_INTERNAL_ERROR as INTERNAL_ERROR, espeak_ERROR_EE_NOT_FOUND as NOT_FOUND,
};

use super::EspeakParam;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("espeak operation failed: {}", espeak_error_msg(*.0))]
    Espeak(espeak_ERROR),

    #[error("no voices available")]
    NoVoicesAvailable,

    #[error("invalid value for '{0:?}': {1}")]
    InvalidParamValue(EspeakParam, u32),

    #[error(transparent)]
    NullPointer(#[from] NulError),

    #[error(transparent)]
    InvalidUtf8(#[from] Utf8Error),
}

fn espeak_error_msg(code: espeak_ERROR) -> &'static str {
    match code {
        INTERNAL_ERROR => "internal error",
        BUFFER_FULL => "buffer full",
        NOT_FOUND => "not found",
        _ => "unknown error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn espeak_variant_to_string_returns_expected() {
        assert_eq!(
            Error::Espeak(INTERNAL_ERROR).to_string(),
            "espeak operation failed: internal error"
        );
        assert_eq!(
            Error::Espeak(BUFFER_FULL).to_string(),
            "espeak operation failed: buffer full"
        );
        assert_eq!(
            Error::Espeak(NOT_FOUND).to_string(),
            "espeak operation failed: not found"
        );
        assert_eq!(
            Error::Espeak(10).to_string(),
            "espeak operation failed: unknown error"
        );
    }

    #[test]
    fn invalid_param_value_variant_to_string_returns_expected() {
        assert_eq!(
            Error::InvalidParamValue(EspeakParam::Amplitude, 666).to_string(),
            "invalid value for 'Amplitude': 666"
        );
        assert_eq!(
            Error::InvalidParamValue(EspeakParam::Pitch, 666).to_string(),
            "invalid value for 'Pitch': 666"
        );
        assert_eq!(
            Error::InvalidParamValue(EspeakParam::PitchRange, 666).to_string(),
            "invalid value for 'PitchRange': 666"
        );
        assert_eq!(
            Error::InvalidParamValue(EspeakParam::Speed, 666).to_string(),
            "invalid value for 'Speed': 666"
        );
        assert_eq!(
            Error::InvalidParamValue(EspeakParam::WordGap, 666).to_string(),
            "invalid value for 'WordGap': 666"
        );
    }
}
