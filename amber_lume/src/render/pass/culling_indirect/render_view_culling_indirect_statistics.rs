use bytemuck::{Pod, Zeroable};

pub const MAIN_CULLING_META_NAME: &str = "main_culling_indirect.views";
pub const CASCADE_CULLING_META_NAME: &str = "cascade_culling_indirect.views";
pub const CASCADE_BLEND_CULLING_META_NAME: &str = "cascade_blend_culling_indirect.views";
pub const TRANSPARENT_CULLING_META_NAME: &str = "transparent_culling_indirect.views";

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct CullingIndirectRenderViewStatisticsGPU {
    pub submeshes_rendered: u32,
    pub submeshes_culled: u32,

    pub _pad0: [u32; 2],
}
