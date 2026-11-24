use bytemuck::{Pod, Zeroable};
use math::Mat4;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct ShadowCascadeGPU {
    pub view_projection: Mat4,
    pub split: f32,
    pub world_radius: f32,
    _pad0: [u32; 2],
}

impl Default for ShadowCascadeGPU {
    fn default() -> Self {
        Self {
            view_projection: Mat4::ZERO,
            split: 0.0,
            world_radius: 0.0,

            _pad0: [0; 2],
        }
    }
}
