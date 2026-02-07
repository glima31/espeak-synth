use std::ffi::CString;
use std::num::NonZeroU32;
use std::path::Path;

use espeak_sys::*;

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
    fn new_invalid_data_dir_panics() {
        let invalid_dir = Path::new("./invalid");
        let _ = EspeakSynth::new(invalid_dir);
    }
}
