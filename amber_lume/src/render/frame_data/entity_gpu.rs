use bytemuck::{Pod, Zeroable};
use glam::Mat4;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct EntityGPU {
    pub transform_matrix: [[f32; 4]; 4],
    pub mesh_index: u32,
    pub bone_transform_offset: u32,
    _pad0: [u32; 2],
}

impl EntityGPU {
    pub const BONE_TRANSFORM_NONE: u32 = u32::MAX;

    pub fn create(
        transform_matrix: Mat4,
        mesh_index: u32,
        bone_transform_offset: u32,
    ) -> Self {
        Self {
            transform_matrix: transform_matrix.to_cols_array_2d(),
            mesh_index,
            bone_transform_offset,
            _pad0: [0; 2],
        }
    }
}
