use std::ffi::{CStr, CString, c_void};
use std::num::NonZeroU32;
use std::path::Path;
use std::ptr;

use espeak_sys::*;

mod callback;
mod error;
mod parameter;

pub use error::*;
pub use parameter::*;

pub struct EspeakSynth {
    sample_rate: NonZeroU32,
}

impl Default for EspeakSynth {
    fn default() -> Self {
        let data_dir = env!("ESPEAK_NG_DATA_DIR");
        Self::new(Path::new(data_dir))
    }
}

impl Drop for EspeakSynth {
    fn drop(&mut self) {
        unsafe { espeak_Terminate() };
    }
}

impl EspeakSynth {
    pub fn new(data_dir: &Path) -> Self {
        if !data_dir.exists() {
            panic!(
                "espeak-ng-data directory does not exist: {}",
                data_dir.display()
            )
        }

        let data_dir = CString::new(data_dir.to_str().unwrap()).unwrap();
        let sample_rate = unsafe {
            espeak_Initialize(
                espeak_AUDIO_OUTPUT_AUDIO_OUTPUT_SYNCHRONOUS,
                0,
                data_dir.as_ptr(),
                0,
            )
        };

        assert!(
            sample_rate > 0,
            "Espeak initialization failed with EE_INTERNAL_ERROR"
        );

        unsafe {
            espeak_SetSynthCallback(Some(callback::synth_callback));
        };

        Self {
            sample_rate: NonZeroU32::new(sample_rate as u32).unwrap(),
        }
    }

    pub fn sample_rate(&self) -> NonZeroU32 {
        self.sample_rate
    }

    pub fn synthesize(&self, text: &str, audio_buffer: &mut Vec<i16>) -> Result<(), Error> {
        let text = CString::new(text)?;
        let result = unsafe {
            espeak_Synth(
                text.as_ptr().cast(),
                text.as_bytes_with_nul().len(),
                0,
                espeak_POSITION_TYPE_POS_WORD,
                0,
                0,
                ptr::null_mut(),
                audio_buffer as *mut Vec<i16> as *mut c_void,
            )
        };

        if result != espeak_ERROR_EE_OK {
            return Err(Error::Espeak(result));
        }

        Ok(())
    }

    pub fn available_voices(&self) -> Result<Vec<String>, Error> {
        let voices_ptr = unsafe { espeak_ListVoices(ptr::null_mut()) };
        if voices_ptr.is_null() {
            return Err(Error::NoVoicesAvailable);
        }

        let mut voices: Vec<String> = Vec::new();
        let mut i = 0;

        loop {
            let voice = unsafe { *voices_ptr.add(i) };
            if voice.is_null() {
                break;
            }

            unsafe {
                let voice = &*voice;
                if !voice.name.is_null() {
                    let name = CStr::from_ptr(voice.name).to_str()?.to_string();
                    voices.push(name);
                }
            }

            i += 1;
        }

        Ok(voices)
    }

    pub fn set_voice(&self, voice: &str) -> Result<(), Error> {
        let s = CString::new(voice)?;

        let result = unsafe { espeak_SetVoiceByName(s.as_ptr()) };
        if result != espeak_ERROR_EE_OK {
            return Err(Error::Espeak(result));
        }

        Ok(())
    }

    pub fn set_parameter(&self, param: EspeakParam, value: u32) -> Result<(), Error> {
        validate_param_value(param, value)?;

        let result = unsafe { espeak_SetParameter(param as _, value as _, 0) };
        if result != espeak_ERROR_EE_OK {
            return Err(Error::Espeak(result));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hound::WavReader;

    const REFERENCE_OUTPUT_WAV: &str = "testdata/dies_ist_ein_test.wav";
    const REFERENCE_TEXT: &str = "Dies ist ein Test";
    const REFERENCE_VOICE: &str = "German";
    const REFERENCE_PITCH: u32 = 40;
    const REFERNECE_SPEED: u32 = 80;

    #[test]
    fn default_initializes_espeak() {
        let espeak = EspeakSynth::default();
        assert!(espeak.sample_rate.get() >= 22050);
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
        assert!(matches!(err, Error::Espeak(code) if code == espeak_ERROR_EE_NOT_FOUND));
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
        let reference_samples: Vec<i16> =
            reference_wav.into_samples().map(|s| s.unwrap()).collect();

        assert_eq!(buffer, reference_samples);
    }
}
