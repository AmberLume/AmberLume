use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct ShadowCascadeGPU {
    pub light_space_view_projection: [[f32; 4]; 4],
    pub split: f32,
    pub world_radius: f32,
    _pad0: [u32; 2],
}

impl Default for ShadowCascadeGPU {
    fn default() -> Self {
        Self {
            light_space_view_projection: [[0.0; 4]; 4],
            split: 0.0,
            world_radius: 0.0,

            _pad0: [0; 2],
        }
    }
}
