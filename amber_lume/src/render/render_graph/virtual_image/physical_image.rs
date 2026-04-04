use ash::vk::{Extent2D, Image, ImageSubresourceRange, ImageView};

#[derive(Clone)]
pub struct PhysicalImage {
    pub image: Image,
    pub image_view: ImageView,
    pub layers: Vec<ImageView>,
    pub extent: Extent2D,
    pub subresource_range: ImageSubresourceRange,
}
