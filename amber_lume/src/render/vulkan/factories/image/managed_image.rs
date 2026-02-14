use ash::vk::{
    Extent3D, Format, Image, ImageAspectFlags, ImageSubresourceRange, ImageTiling, ImageType,
    ImageUsageFlags, ImageView, ImageViewType, SampleCountFlags, SharingMode,
};
use gpu_allocator::vulkan::Allocation;

#[derive(Debug)]
pub struct ManagedImage {
    pub label: String,

    pub image_description: ImageDescription,
    pub image_view_description: ImageViewDescription,

    pub image: Image,
    pub image_view: ImageView,

    pub image_subresource_range: ImageSubresourceRange,
    pub allocation: Allocation,
}

#[derive(Debug)]
pub struct ImageDescription {
    pub image_type: ImageType,
    pub format: Format,
    pub extent: Extent3D,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub samples: SampleCountFlags,
    pub tiling: ImageTiling,
    pub usage: ImageUsageFlags,
    pub sharing_mode: SharingMode,
}

#[derive(Debug)]
pub struct ImageViewDescription {
    pub image_view_type: ImageViewType,
    pub image_aspect_flags: ImageAspectFlags,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}
