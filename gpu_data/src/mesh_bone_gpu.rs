use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct MeshBoneGPU {
    pub inverse_bind_matrix: [[f32; 4]; 4],

    pub bounds_min: [f32; 4],
    pub bounds_max: [f32; 4],
}

impl MeshBoneGPU {
    pub fn create(inverse_bind_matrix: [[f32; 4]; 4], bounds: [f32; 6]) -> Self {
        Self {
            inverse_bind_matrix,

            bounds_min: [bounds[0], bounds[1], bounds[2], 0.0],
            bounds_max: [bounds[3], bounds[4], bounds[5], 0.0],
        }
    }
}
