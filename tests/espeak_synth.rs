use hound::WavReader;
use std::path::Path;

use espeak_synth::*;

const REFERENCE_OUTPUT_WAV: &str = "testdata/dies_ist_ein_test.wav";
const REFERENCE_TEXT: &str = "Dies ist ein Test";
const REFERENCE_VOICE: &str = "German";
const REFERENCE_PITCH: u32 = 40;
const REFERNECE_SPEED: u32 = 80;

#[test]
fn default_initializes_espeak() {
    let espeak = EspeakSynth::default();
    assert!(espeak.sample_rate().get() >= 22050);
}

#[test]
#[should_panic = "espeak-ng-data directory does not exist: ./invalid"]
fn new_with_non_existent_data_dir_panics() {
    let non_existent = Path::new("./invalid");
    let _ = EspeakSynth::new(non_existent);
}

#[test]
fn available_voices_valid_data_dir_result_contains_expected_voices() {
    let espeak = EspeakSynth::default();
    let voices = espeak.available_voices().unwrap();
    assert!(voices.contains(&"German".to_owned()));
    assert!(voices.contains(&"English (Great Britain)".to_owned()));
}

#[test]
fn set_voice_valid_returns_ok() {
    let espeak = EspeakSynth::default();
    let result = espeak.set_voice("German");
    assert!(result.is_ok());
}

#[test]
fn set_voice_invalid_returns_ee_not_found_err() {
    let espeak = EspeakSynth::default();
    let err = espeak.set_voice("Invalid").unwrap_err();
    assert!(matches!(err, Error::Espeak(code) if code == 2));
}

#[test]
fn set_parameter_valid_case_returns_ok() {
    let espeak = EspeakSynth::default();
    let result = espeak.set_parameter(EspeakParam::Amplitude, 100);
    assert!(result.is_ok());
}

#[test]
fn set_parameter_invalid_returns_err() {
    let espeak = EspeakSynth::default();
    let err = espeak
        .set_parameter(EspeakParam::Amplitude, 101)
        .unwrap_err();
    assert!(
        matches!(err, Error::InvalidParamValue(p, v) if p == EspeakParam::Amplitude && v == 101)
    );
}

#[test]
fn voice_returns_none_if_not_explicitely_set() {
    let espeak = EspeakSynth::default();
    let curr = espeak.voice().unwrap();
    assert!(curr.is_none());
}

#[test]
fn voice_returns_new_value_after_set_voice() {
    let espeak = EspeakSynth::default();
    espeak.set_voice("German").unwrap();

    let new = espeak.voice().unwrap();
    assert_eq!(new.unwrap(), "German");
}

#[test]
fn parameter_default_returns_expected_defaults() {
    let espeak = EspeakSynth::default();
    let expected_defaults = vec![
        (EspeakParam::Amplitude, 100),
        (EspeakParam::Pitch, 50),
        (EspeakParam::PitchRange, 50),
        (EspeakParam::Speed, 175),
        (EspeakParam::WordGap, 0),
    ];

    for (param, expected) in expected_defaults {
        let result = espeak.parameter_default(param);
        assert_eq!(result, expected);
    }
}

#[test]
fn parameter_default_returns_same_values_after_parameter_change() {
    let espeak = EspeakSynth::default();
    let test_cases = vec![
        (EspeakParam::Amplitude, 50, 100),
        (EspeakParam::Pitch, 100, 50),
        (EspeakParam::PitchRange, 90, 50),
        (EspeakParam::Speed, 200, 175),
        (EspeakParam::WordGap, 50, 0),
    ];

    for (param, new_value, expected) in test_cases {
        espeak.set_parameter(param, new_value).unwrap();
        let result = espeak.parameter_default(param);
        assert_eq!(result, expected);
    }
}

#[test]
fn parameter_current_returns_new_values_after_parameter_change() {
    let espeak = EspeakSynth::default();
    let test_cases = vec![
        (EspeakParam::Amplitude, 50),
        (EspeakParam::Pitch, 100),
        (EspeakParam::PitchRange, 90),
        (EspeakParam::Speed, 200),
        (EspeakParam::WordGap, 50),
    ];

    for (param, new_value) in test_cases {
        espeak.set_parameter(param, new_value).unwrap();
        let result = espeak.parameter_current(param);
        assert_eq!(result, new_value);
    }
}

#[test]
fn synthesize_with_default_settings_works() {
    let espeak = EspeakSynth::default();
    let mut buffer = Vec::new();

    espeak.synthesize("test", &mut buffer).unwrap();
    assert!(!buffer.is_empty());
}

#[test]
fn synthesize_known_settings_result_matches_reference() {
    let espeak = EspeakSynth::default();
    let mut buffer = Vec::new();

    espeak.set_voice(REFERENCE_VOICE).unwrap();
    espeak
        .set_parameter(EspeakParam::Pitch, REFERENCE_PITCH)
        .unwrap();
    espeak
        .set_parameter(EspeakParam::Speed, REFERNECE_SPEED)
        .unwrap();

    espeak.synthesize(REFERENCE_TEXT, &mut buffer).unwrap();
    assert!(!buffer.is_empty());

    let reference_wav = WavReader::open(REFERENCE_OUTPUT_WAV).unwrap();
    let reference_samples: Vec<i16> = reference_wav.into_samples().map(|s| s.unwrap()).collect();

    assert_eq!(buffer, reference_samples);
}
