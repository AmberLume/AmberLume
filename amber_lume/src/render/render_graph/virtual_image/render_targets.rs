use crate::render::render_graph::virtual_image::virtual_image::VirtualImage;

pub struct ColorTarget {
    pub image: VirtualImage,
    pub clear: Option<[f32; 4]>,
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
