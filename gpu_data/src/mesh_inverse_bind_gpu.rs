use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct MeshInverseBindGPU {
    pub inverse_bind_matrix: [[f32; 4]; 4],
}

impl MeshInverseBindGPU {
    pub fn create(inverse_bind_matrix: [[f32; 4]; 4]) -> Self {
        Self {
            inverse_bind_matrix,
        }
    }
}
