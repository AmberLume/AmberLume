use ash::vk::{Format, ImageUsageFlags};
use crate::render::factories::image::image_view_description::ImageViewDescription;
use crate::render::render_graph::virtual_image::image_size::ImageSize;
use crate::resources::binding_layout::descriptor_set_manager::GlobalDescriptorSetBindings;
use crate::resources::sampler_registry::SamplerType;

#[derive(Clone, Copy)]
pub struct ImageBlueprint {
    pub size: ImageSize,
    pub format: Format,
    pub usage: ImageUsageFlags,
    pub image_view_description: ImageViewDescription,
    pub descriptor: Option<(GlobalDescriptorSetBindings, SamplerType)>,
}
