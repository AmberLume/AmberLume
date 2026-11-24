use bytemuck::{Pod, Zeroable};
use math::Vec3;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SubmeshBoundsGPU {
    pub min: Vec3,
    pub max: Vec3,
}

impl SubmeshBoundsGPU {
    pub fn create(bounds: [f32; 6]) -> Self {
        Self {
            min: Vec3::new(bounds[0], bounds[1], bounds[2]),
            max: Vec3::new(bounds[3], bounds[4], bounds[5]),
        }
    }
}
