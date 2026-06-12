use ash::vk::{Extent2D, Format, Image, ImageSubresourceRange, ImageView};

#[derive(Clone)]
pub struct PhysicalImageDescriptors {
    pub full: Option<u32>,
    pub storage_mips: Option<Vec<u32>>,
}

#[derive(Clone)]
pub struct PhysicalImage {
    pub image: Image,
    pub image_view: ImageView,
    pub extent: Extent2D,
    pub format: Format,
    pub subresource_range: ImageSubresourceRange,
    pub descriptors: PhysicalImageDescriptors,
}
