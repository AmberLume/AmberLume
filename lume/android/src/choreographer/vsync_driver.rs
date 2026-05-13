use crate::choreographer::choreographer::{AChoreographer, GetInstanceFn, PostFrameCallback64Fn};
use crate::choreographer::utils::{dlopen, lookup, RTLD_DEFAULT, RTLD_NOW};
use std::ffi::{c_void, CString};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct VsyncDriver {
    handle: *mut AChoreographer,
    post_frame_callback: PostFrameCallback64Fn,
    frame_pending: AtomicBool,
}

unsafe impl Send for VsyncDriver {}
unsafe impl Sync for VsyncDriver {}

impl VsyncDriver {
    pub fn create() -> Option<&'static Self> {
        unsafe {
            let lib_name = CString::new("libandroid.so").ok()?;
            let lib_handle = dlopen(lib_name.as_ptr(), RTLD_NOW);
            let lookup_handle = if lib_handle.is_null() {
                RTLD_DEFAULT
            } else {
                lib_handle
            };

            let get_instance: GetInstanceFn = lookup(lookup_handle, "AChoreographer_getInstance")?;
            let post_frame_callback = lookup(lookup_handle, "AChoreographer_postFrameCallback64")?;

            let handle = get_instance();
            if handle.is_null() {
                return None;
            }

            let driver = Box::leak(Box::new(Self {
                handle,
                post_frame_callback,
                frame_pending: AtomicBool::new(false),
            }));
            driver.request_next_frame();

            Some(driver)
        }
    }

    pub fn consume_frame(&self) -> bool {
        self.frame_pending.swap(false, Ordering::AcqRel)
    }

    pub fn request_next_frame(&self) {
        let userdata = self as *const Self as *mut c_void;
        unsafe {
            (self.post_frame_callback)(self.handle, Self::vsync_callback, userdata);
        }
    }

    unsafe extern "C" fn vsync_callback(_frame_time_nanos: i64, data: *mut c_void) {
        let driver = unsafe { &*(data as *const Self) };

        driver.frame_pending.store(true, Ordering::Release);
    }
}
