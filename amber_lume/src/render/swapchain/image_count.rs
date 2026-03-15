use ash::vk::SurfaceCapabilitiesKHR;
use tracing::info;

pub fn get_image_count(surface_capabilities: &SurfaceCapabilitiesKHR) -> u32 {
    let mut desired = surface_capabilities.min_image_count + 1;

    let max_image_count = surface_capabilities.max_image_count;

    if max_image_count > 0 && desired > max_image_count {
        desired = max_image_count;
    }

    info!("Swapchain image count: {}", desired);

    desired
}
