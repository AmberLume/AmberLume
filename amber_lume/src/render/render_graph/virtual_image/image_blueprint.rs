use ash::vk::{Format, ImageUsageFlags};
use crate::render::factories::image::image_view_description::ImageViewDescription;
use crate::render::render_graph::virtual_image::image_size::ImageSize;

#[derive(Clone, Copy)]
pub struct ImageBlueprint {
    pub size: ImageSize,
    pub format: Format,
    pub usage: ImageUsageFlags,
    pub image_view_description: ImageViewDescription,
}
