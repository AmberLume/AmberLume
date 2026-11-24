use ash::vk::SurfaceCapabilitiesKHR;

pub(crate) struct SurfaceImageCount;

impl SurfaceImageCount {
    pub fn select(capabilities: &SurfaceCapabilitiesKHR, image_count: u32) -> u32 {
        let image_count = image_count.max(capabilities.min_image_count);

        match capabilities.max_image_count {
            0 => image_count,
            max => image_count.min(max),
        }
    }
}
