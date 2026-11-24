use bytemuck::{Pod, Zeroable};
use math::Mat4;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct EntityGPU {
    pub transform_matrix: Mat4,
    pub previous_transform_matrix: Mat4,
    pub mesh_index: u32,
    _pad0: [u32; 3],
}

impl EntityGPU {
    pub fn create(
        transform_matrix: Mat4,
        previous_transform_matrix: Mat4,
        mesh_index: u32,
    ) -> Self {
        Self {
            transform_matrix,
            previous_transform_matrix,
            mesh_index,
            _pad0: [0; 3],
        }
    }
}
