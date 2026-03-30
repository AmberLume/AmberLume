use ash::vk::{Extent3D, Format, Image, ImageAspectFlags, ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageView, ImageViewType, SampleCountFlags, SharingMode};
use gpu_allocator::vulkan::Allocation;

#[derive(Debug)]
pub struct ManagedImage {
    pub label: String,

    pub image_description: ImageDescription,
    pub image_view_description: ImageViewDescription,

    pub image: Image,
    pub image_view: ImageView,
    pub image_view_layers: Vec<ImageView>,

    pub image_subresource_range: ImageSubresourceRange,
    pub allocation: Allocation,
}

#[derive(Clone, Debug)]
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

impl ImageDescription {
    pub fn default(
        format: Format,
        extent: Extent3D,
    ) -> Self {
        Self {
            image_type: ImageType::TYPE_2D,
            format,
            extent,
            mip_levels: 1,
            array_layers: 1,
            samples: SampleCountFlags::TYPE_1,
            tiling: ImageTiling::OPTIMAL,
            usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
            sharing_mode: SharingMode::EXCLUSIVE,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ImageViewDescription {
    pub image_view_type: ImageViewType,
    pub image_aspect_flags: ImageAspectFlags,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
    
    pub layered: bool,
}

impl ImageViewDescription {
    pub fn default_2d_color() -> Self {
        Self {
            image_view_type: ImageViewType::TYPE_2D,
            image_aspect_flags: ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,

            layered: false,
        }
    }

    pub fn default_2d_array_depth(layer_count: u32) -> Self {
        Self {
            image_view_type: ImageViewType::TYPE_2D_ARRAY,
            image_aspect_flags: ImageAspectFlags::DEPTH,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count,

            layered: true,
        }
    }
}
