use ash::vk::{Extent2D, Format, Image, ImageSubresourceRange, ImageView};

#[derive(Clone)]
pub struct PhysicalImage {
    pub image: Image,
    pub image_view: ImageView,
    pub layers: Vec<ImageView>,
    pub extent: Extent2D,
    pub format: Format,
    pub subresource_range: ImageSubresourceRange,
    pub descriptor_id: Option<u32>,
}
