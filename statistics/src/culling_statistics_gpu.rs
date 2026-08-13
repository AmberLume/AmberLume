use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct CullingIndirectRequestStatisticsGPU {
    pub submeshes_rendered: u32,
    pub submeshes_culled: u32,
    pub submeshes_dropped: u32,

    pub _pad0: u32,
}
