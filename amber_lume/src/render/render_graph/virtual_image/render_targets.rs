use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;

#[derive(Clone, Copy)]
pub enum ClearColor {
    Float([f32; 4]),
    Uint([u32; 4]),
}

pub struct ColorTarget {
    pub image: VirtualImage,
    pub mip: Option<u32>,
    pub clear: Option<ClearColor>,
}

pub struct DepthTarget {
    pub image: VirtualImage,
    pub clear: Option<f32>,
}

pub struct RenderTargets {
    pub color: Vec<ColorTarget>,
    pub depth: Option<DepthTarget>,
    pub view_mask: u32,
}
