use ash::vk::{Extent2D, Format, Image, ImageSubresourceRange, ImageView};
use crate::resources::store::providers::resource_provider::ResourceId;

#[derive(Clone)]
pub struct PhysicalImage {
    pub image: Image,
    pub image_view: ImageView,
    pub extent: Extent2D,
    pub format: Format,
    pub subresource_range: ImageSubresourceRange,
    pub descriptors: PhysicalImageDescriptors,
}

#[derive(Clone)]
pub struct PhysicalImageDescriptors {
    pub full: Option<ResourceId>,
    pub storage_mips: Option<Vec<ResourceId>>,
}
