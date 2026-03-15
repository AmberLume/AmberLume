use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct RawGpuRenderStatistics {
    pub render_time: StageMeasurement,

    pub submeshes_rendered: u32,
    pub submeshes_culled: u32,

    pub _pad0: [u32; 2],
}

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct StageMeasurement {
    pub start: u64,
    pub end: u64,
}
