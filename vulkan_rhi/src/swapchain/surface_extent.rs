use ash::vk::{Extent2D, SurfaceCapabilitiesKHR};
use explicit_rhi::WindowSize;

pub(crate) struct SurfaceExtent;

impl SurfaceExtent {
    pub fn select(capabilities: &SurfaceCapabilitiesKHR, size: WindowSize) -> Extent2D {
        if size.width == 0 || size.height == 0 {
            return Extent2D { width: 0, height: 0 };
        }

        if capabilities.current_extent.width != u32::MAX {
            return capabilities.current_extent;
        }

        let width = size.width.clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width);
        let height = size.height.clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height);

        Extent2D { width, height }
    }
}
