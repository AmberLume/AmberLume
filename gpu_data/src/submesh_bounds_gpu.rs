use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SubmeshBoundsGPU {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl SubmeshBoundsGPU {
    pub fn create(bounds: [f32; 6]) -> Self {
        Self {
            min: [bounds[0], bounds[1], bounds[2]],
            max: [bounds[3], bounds[4], bounds[5]],
        }
    }
}
