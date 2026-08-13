use std::ptr::NonNull;
use gpu::SurfaceProvider;
use android_activity::AndroidApp;
use raw_window_handle::{AndroidDisplayHandle, AndroidNdkWindowHandle, RawDisplayHandle, RawWindowHandle};

pub struct AndroidSurfaceProvider {
    android_app: AndroidApp,
}

impl AndroidSurfaceProvider {
    pub fn new(android_app: AndroidApp) -> Self {
        Self {
            android_app,
        }
    }
}

impl SurfaceProvider for AndroidSurfaceProvider {
    fn handles(&self) -> (RawDisplayHandle, RawWindowHandle) {
        let native_window = self
            .android_app
            .native_window()
            .expect("native_window unavailable — surface not initialized");

        let ptr = NonNull::new(native_window.ptr().as_ptr() as *mut _)
            .expect("ANativeWindow pointer is null");

        let raw_window_handle = RawWindowHandle::AndroidNdk(
            AndroidNdkWindowHandle::new(ptr),
        );
        let raw_display_handle = RawDisplayHandle::Android(AndroidDisplayHandle::new());

        (raw_display_handle, raw_window_handle)
    }

    fn size(&self) -> (u32, u32) {
        let native_window = self.android_app.native_window().expect("native_window unavailable");

        (native_window.width() as u32, native_window.height() as u32)
    }
}
