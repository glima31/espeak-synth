use espeak_sys::espeak_EVENT;
use std::ffi::{c_int, c_short};

pub(crate) unsafe extern "C" fn synth_callback(
    wav: *mut c_short,
    num_samples: c_int,
    events: *mut espeak_EVENT,
) -> c_int {
    if wav.is_null() || num_samples <= 0 || events.is_null() {
        return 0;
    }

    let user_data = unsafe { (*events).user_data };
    if user_data.is_null() {
        return 0;
    }

    let buffer = unsafe { &mut *(user_data as *mut Vec<i16>) };
    let slice = unsafe { std::slice::from_raw_parts(wav, num_samples as usize) };
    buffer.extend_from_slice(slice);

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    fn create_mock_event(user_data: *mut std::ffi::c_void) -> espeak_EVENT {
        espeak_EVENT {
            type_: 0,
            unique_identifier: 0,
            text_position: 0,
            length: 0,
            audio_position: 0,
            sample: 0,
            user_data,
            id: espeak_sys::espeak_EVENT__bindgen_ty_1 { number: 0 },
        }
    }

    #[test]
    fn appends_samples_to_buffer() {
        let mut buffer: Vec<i16> = Vec::new();
        let mut wav_data: Vec<c_short> = vec![1, 2, 3, 4, 5, 6];
        let mut event = create_mock_event(&mut buffer as *mut Vec<i16> as *mut _);

        let result =
            unsafe { synth_callback(wav_data.as_mut_ptr(), wav_data.len() as c_int, &mut event) };

        assert_eq!(result, 0);
        assert_eq!(buffer, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn null_wav_returns_zero() {
        let mut buffer: Vec<i16> = Vec::new();
        let mut event = create_mock_event(&mut buffer as *mut Vec<i16> as *mut _);

        let result = unsafe { synth_callback(ptr::null_mut(), 10, &mut event) };

        assert_eq!(result, 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn null_user_data_returns_zero() {
        let mut wav_data: Vec<c_short> = vec![1, 2, 3];
        let mut event = create_mock_event(ptr::null_mut());

        let result =
            unsafe { synth_callback(wav_data.as_mut_ptr(), wav_data.len() as c_int, &mut event) };

        assert_eq!(result, 0);
    }
}
