use std::ffi::{c_char, c_int, c_void, CString};
use std::mem::transmute_copy;
use std::ptr;

unsafe extern "C" {
    pub fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

pub const RTLD_NOW: c_int = 2;
pub const RTLD_DEFAULT: *mut c_void = ptr::null_mut();

pub unsafe fn lookup<T>(handle: *mut c_void, name: &str) -> Option<T> {
    let cstr = CString::new(name).ok()?;
    let symbol = unsafe { dlsym(handle, cstr.as_ptr()) };

    if symbol.is_null() {
        None
    } else {
        Some(unsafe { transmute_copy::<*mut c_void, T>(&symbol) })
    }
}
