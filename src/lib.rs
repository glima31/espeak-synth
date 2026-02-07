use std::ffi::{CStr, CString};
use std::num::NonZeroU32;
use std::path::Path;
use std::ptr;

use espeak_sys::*;

mod error;
pub use error::*;

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

        Self {
            sample_rate: NonZeroU32::new(sample_rate as u32).unwrap(),
        }
    }

    pub fn sample_rate(&self) -> NonZeroU32 {
        self.sample_rate
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

    fn set_voice(&self, voice: &str) -> Result<(), Error> {
        let s = CString::new(voice)?;

        let result = unsafe { espeak_SetVoiceByName(s.as_ptr()) };
        if result != espeak_ERROR_EE_OK {
            return Err(Error::Espeak(result));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
