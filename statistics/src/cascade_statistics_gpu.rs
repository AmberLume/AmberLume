use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct CascadeStatisticsGPU {
    pub range_start: f32,
    pub range_end: f32,
    pub world_radius: f32,
    pub z_max: f32,
}
