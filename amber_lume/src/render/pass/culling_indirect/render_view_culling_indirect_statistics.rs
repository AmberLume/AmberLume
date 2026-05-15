use bytemuck::{Pod, Zeroable};

pub const MAIN_CULLING_META_NAME: &str = "main_culling_indirect.views";
pub const CASCADE_CULLING_META_NAME: &str = "cascade_culling_indirect.views";

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct CullingIndirectRenderViewStatisticsGPU {
    pub submeshes_rendered: u32,
    pub submeshes_culled: u32,

    pub _pad0: [u32; 2],
}

pub struct CullingIndirectRenderViewStatistics {
    pub submeshes_rendered: u32,
    pub submeshes_culled: u32,
}

pub struct CullingIndirectStatistics {
    pub render_views: Vec<CullingIndirectRenderViewStatistics>,
}
