use amber_lume::gpu::SurfaceProvider;
use std::sync::Arc;
use winit::raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use winit::window::Window;

pub struct VulkanSurfaceProvider {
    window: Arc<Window>,
}

impl VulkanSurfaceProvider {
    pub fn new(window: Arc<Window>) -> Self {
        Self { window }
    }
}

impl SurfaceProvider for VulkanSurfaceProvider {
    fn handles(&self) -> (RawDisplayHandle, RawWindowHandle) {
        let raw_display_handle = self.window.display_handle().unwrap().as_raw();
        let raw_window_handle = self.window.window_handle().unwrap().as_raw();

        (raw_display_handle, raw_window_handle)
    }

    fn size(&self) -> (u32, u32) {
        let inner_size = self.window.inner_size();

        (inner_size.width, inner_size.height)
    }
}
