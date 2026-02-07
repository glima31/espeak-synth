use super::Error;

pub const MAX_AMPLITUDE: u32 = 100;

pub const MAX_PITCH: u32 = 100;

pub const MAX_PITCH_RANGE: u32 = 100;

pub const MAX_WORD_GAP: u32 = 100;

pub const MIN_SPEED: u32 = 80;

pub const MAX_SPEED: u32 = 450;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EspeakParam {
    Amplitude = 2,
    Pitch = 3,
    PitchRange = 4,
    Speed = 1,
    WordGap = 7,
}

pub(crate) fn validate_param_value(param: EspeakParam, value: u32) -> Result<(), Error> {
    let (min, max) = match param {
        EspeakParam::Amplitude => (0, MAX_AMPLITUDE),
        EspeakParam::Pitch => (0, MAX_PITCH),
        EspeakParam::PitchRange => (0, MAX_PITCH_RANGE),
        EspeakParam::Speed => (MIN_SPEED, MAX_SPEED),
        EspeakParam::WordGap => (0, MAX_WORD_GAP),
    };

    if !(min..=max).contains(&value) {
        return Err(Error::InvalidParamValue(param, value));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_param_value_valid_cases_return_ok() {
        let test_cases = vec![
            (EspeakParam::Amplitude, 0),
            (EspeakParam::Amplitude, 50),
            (EspeakParam::Amplitude, 100),
            (EspeakParam::Pitch, 0),
            (EspeakParam::Pitch, 50),
            (EspeakParam::Pitch, 100),
            (EspeakParam::PitchRange, 0),
            (EspeakParam::PitchRange, 50),
            (EspeakParam::PitchRange, 100),
            (EspeakParam::Speed, 80),
            (EspeakParam::Speed, 200),
            (EspeakParam::Speed, 450),
            (EspeakParam::WordGap, 0),
            (EspeakParam::WordGap, 50),
            (EspeakParam::WordGap, 100),
        ];

        for (param, val) in test_cases {
            assert!(
                validate_param_value(param, val).is_ok(),
                "({param:?}, {val}) returned err"
            );
        }
    }

    #[test]
    fn validate_param_value_invalid_cases_return_err() {
        let test_cases = vec![
            (EspeakParam::Amplitude, 101),
            (EspeakParam::Pitch, 101),
            (EspeakParam::PitchRange, 101),
            (EspeakParam::Speed, 79),
            (EspeakParam::Speed, 451),
            (EspeakParam::WordGap, 101),
        ];

        for (param, val) in test_cases {
            let result = validate_param_value(param, val);
            assert!(result.is_err(), "({param:?}, {val}) returned err");
            assert!(
                matches!(result.unwrap_err(), Error::InvalidParamValue(p, v) if p == param && v == val)
            );
        }
    }
}
