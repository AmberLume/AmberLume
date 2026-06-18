use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageSubresource {
    pub image: VirtualImage,
    pub mip: Option<u32>,
}
