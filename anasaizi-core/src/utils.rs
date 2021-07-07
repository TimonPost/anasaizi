use std::{
    ffi::{CStr, CString, IntoStringError},
    os::raw::c_char,
};

pub fn vk_to_cstr(raw_string_array: &[c_char]) -> CString {
    // Implementation 2
    unsafe {
        let pointer = raw_string_array.as_ptr() as *mut c_char;
        CString::from_raw(pointer)
    }
}

pub fn vk_to_string(raw_string_array: &[c_char]) -> Result<String, IntoStringError> {
    let c_str = unsafe { CStr::from_ptr(raw_string_array.as_ptr()) };
    let c_string = c_str.to_owned();
    let string = c_string.into_string();

    string
}
