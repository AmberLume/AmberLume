use ash::vk::{Format, ImageAspectFlags, ImageCreateFlags, ImageUsageFlags};
use crate::render::factories::image::image_view_description::ImageViewDescription;
use crate::render::render_graph::virtual_image::image_size::ImageSize;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImageBlueprint {
    pub size: ImageSize,
    pub format: Format,
    pub usage: ImageUsageFlags,
    pub array_layers: u32,
    pub image_view_description: ImageViewDescription,
    pub flags: ImageCreateFlags,
    pub sampled: bool,
}

impl ImageBlueprint {
    pub fn color(size: ImageSize, format: Format) -> Self {
        Self {
            size,
            format,
            usage: ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
            array_layers: 1,
            image_view_description: ImageViewDescription::default_2d_color(),
            flags: ImageCreateFlags::empty(),
            sampled: true,
        }
    }

    pub fn storage(size: ImageSize, format: Format) -> Self {
        Self {
            usage: ImageUsageFlags::STORAGE | ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
            ..Self::color(size, format)
        }
    }

    pub fn depth(size: ImageSize, format: Format) -> Self {
        Self {
            usage: ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
            image_view_description: ImageViewDescription {
                image_aspect_flags: ImageAspectFlags::DEPTH,
                ..ImageViewDescription::default_2d_color()
            },
            ..Self::color(size, format)
        }
    }

    pub fn shadow_map(size: ImageSize, format: Format, array_layers: u32) -> Self {
        Self {
            usage: ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
            array_layers,
            image_view_description: ImageViewDescription::default_2d_array_depth(array_layers),
            ..Self::color(size, format)
        }
    }
}
