use crate::virtual_image::virtual_image::VirtualImage;

pub struct DepthTarget {
    pub image: VirtualImage,
    pub clear: Option<f32>,
}
