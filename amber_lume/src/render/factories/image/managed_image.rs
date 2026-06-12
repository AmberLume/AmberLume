use ash::vk::{Image, ImageSubresourceRange, ImageView};
use gpu_allocator::vulkan::Allocation;
use crate::render::factories::image::image_description::ImageDescription;
use crate::render::factories::image::image_view_description::ImageViewDescription;

#[derive(Debug)]
pub struct ManagedImage {
    pub label: String,

    pub image_description: ImageDescription,
    pub image_view_description: ImageViewDescription,

    pub image: Image,
    pub image_view: ImageView,
    pub storage_mip_views: Vec<ImageView>,

    pub image_subresource_range: ImageSubresourceRange,
    pub allocation: Allocation,
}
