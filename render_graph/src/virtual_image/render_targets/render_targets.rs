use crate::virtual_image::render_targets::color_target::ColorTarget;
use crate::virtual_image::render_targets::depth_target::DepthTarget;

pub struct RenderTargets {
    pub color: Vec<ColorTarget>,
    pub depth: Option<DepthTarget>,
    pub view_mask: u32,
}
