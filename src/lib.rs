use libc::c_char;
use std::ffi::CStr;

pub mod common;
pub mod fe_utils;
pub mod include;
pub mod port;

/// Create an owned String from a possilby NULL C-string.
pub fn safe_cstr(ptr: *const c_char) -> Option<String> {
    unsafe {
        if ptr.is_null() {
            None
        } else {
            Some(CStr::from_ptr(ptr).to_str().unwrap().to_owned())
        }
    }
}
