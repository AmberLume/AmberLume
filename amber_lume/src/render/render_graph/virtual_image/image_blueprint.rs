use ash::vk::{Format, ImageUsageFlags};
use crate::render::factories::image::image_view_description::ImageViewDescription;
use crate::render::render_graph::virtual_image::image_size::ImageSize;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImageBlueprint {
    pub size: ImageSize,
    pub format: Format,
    pub usage: ImageUsageFlags,
    pub array_layers: u32,
    pub image_view_description: ImageViewDescription,
    pub sampled: bool,
}
