use ash::vk::{Image, ImageAspectFlags};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ImageRegionKey {
    pub image: Image,
    pub aspect_mask: ImageAspectFlags,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}
