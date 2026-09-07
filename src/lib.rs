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

/// Safe interface to the eSpeak NG TTS engine.
///
/// eSpeak NG keeps global state, so only one instance should exist at a time.
/// Dropping this struct terminates the engine.
pub struct EspeakSynth {
    sample_rate: NonZeroU32,
}

impl Default for EspeakSynth {
    /// Initializes with the voice data produced by this crate's build.
    ///
    /// The path is baked in at compile time, so it only resolves on the machine that
    /// built the crate. Use [`EspeakSynth::new`] for anything you distribute.
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
    /// Initializes the engine with the voice data at `data_dir`.
    ///
    /// # Panics
    ///
    /// Panics if `data_dir` does not exist, or if eSpeak NG fails to initialize.
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

    /// Sample rate eSpeak NG reported at initialization, in Hz.
    pub fn sample_rate(&self) -> NonZeroU32 {
        self.sample_rate
    }

    /// Synthesizes `text`, appending mono `i16` samples to `audio_buffer`.
    ///
    /// Samples are appended rather than replaced, so clear the buffer between calls
    /// if you don't want them to accumulate.
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

    /// Name of the voice that is currently set or `None` if not set.
    pub fn voice(&self) -> Result<Option<String>, Error> {
        let curr_voice_ptr = unsafe { espeak_GetCurrentVoice() };
        if curr_voice_ptr.is_null() {
            return Ok(None);
        }

        let voice = unsafe {
            let curr_voice = &*curr_voice_ptr;
            if curr_voice.name.is_null() {
                return Ok(None);
            }

            CStr::from_ptr(curr_voice.name).to_str()?.to_owned()
        };

        Ok(Some(voice))
    }

    /// Current value of `param`.
    pub fn parameter_current(&self, param: EspeakParam) -> u32 {
        unsafe { espeak_GetParameter(param as _, 1) as u32 }
    }

    /// Default value of `param`.
    pub fn parameter_default(&self, param: EspeakParam) -> u32 {
        unsafe { espeak_GetParameter(param as _, 0) as u32 }
    }

    /// Names of every voice in the loaded voice data.
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

    /// Sets the voice by name, as returned by [`EspeakSynth::available_voices`].
    pub fn set_voice(&self, voice: &str) -> Result<(), Error> {
        let s = CString::new(voice)?;

        let result = unsafe { espeak_SetVoiceByName(s.as_ptr()) };
        if result != espeak_ERROR_EE_OK {
            return Err(Error::Espeak(result));
        }

        Ok(())
    }

    /// Sets `param` to `value`.
    ///
    /// Returns [`Error::InvalidParamValue`] if `value` is outside the accepted range
    /// for that parameter.
    pub fn set_parameter(&self, param: EspeakParam, value: u32) -> Result<(), Error> {
        validate_param_value(param, value)?;

        let result = unsafe { espeak_SetParameter(param as _, value as _, 0) };
        if result != espeak_ERROR_EE_OK {
            return Err(Error::Espeak(result));
        }

        Ok(())
    }
}
