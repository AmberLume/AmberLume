use std::ffi::c_void;

#[repr(C)]
pub struct AChoreographer {
    _private: [u8; 0],
}

pub type FrameCallback = unsafe extern "C" fn(frame_time_nanos: i64, data: *mut c_void);
pub type GetInstanceFn = unsafe extern "C" fn() -> *mut AChoreographer;
pub type PostFrameCallback64Fn = unsafe extern "C" fn(*mut AChoreographer, FrameCallback, *mut c_void);
