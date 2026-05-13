use crate::choreographer::utils::{dlopen, lookup, RTLD_DEFAULT, RTLD_NOW};
use std::ffi::{c_void, CString};

type SetFrameRateFn = unsafe extern "C" fn(*mut c_void, f32, i8) -> i32;

pub struct FrameRateBinding {
    set_frame_rate: SetFrameRateFn,
}

unsafe impl Send for FrameRateBinding {}
unsafe impl Sync for FrameRateBinding {}

impl FrameRateBinding {
    pub fn create() -> Option<&'static Self> {
        unsafe {
            let lib_name = CString::new("libnativewindow.so").ok()?;
            let lib_handle = dlopen(lib_name.as_ptr(), RTLD_NOW);
            let lookup_handle = if lib_handle.is_null() {
                RTLD_DEFAULT
            } else {
                lib_handle
            };

            let set_frame_rate: SetFrameRateFn = lookup(lookup_handle, "ANativeWindow_setFrameRate")?;

            Some(Box::leak(Box::new(Self { set_frame_rate })))
        }
    }

    pub fn set(&self, window: *mut c_void, frame_rate: f32, compatibility: i8) -> i32 {
        unsafe { (self.set_frame_rate)(window, frame_rate, compatibility) }
    }
}
