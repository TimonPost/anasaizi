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

/// Return a `&[u8]` for any sized object passed in.
pub unsafe fn any_as_u8_slice<T: Sized>(any: &T) -> &[u8] {
    let ptr = (any as *const T) as *const u8;
    std::slice::from_raw_parts(ptr, std::mem::size_of::<T>())
}
