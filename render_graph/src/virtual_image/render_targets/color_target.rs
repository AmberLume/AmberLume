use crate::virtual_image::render_targets::clear_color::ClearColor;
use crate::virtual_image::virtual_image::VirtualImage;

pub struct ColorTarget {
    pub image: VirtualImage,
    pub mip: Option<u32>,
    pub clear: Option<ClearColor>,
}
