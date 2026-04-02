use bytemuck::{Pod, Zeroable};

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
