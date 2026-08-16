use bytemuck::{Pod, Zeroable};
use glam::Mat4;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct EntityGPU {
    pub transform_matrix: [[f32; 4]; 4],
    pub previous_transform_matrix: [[f32; 4]; 4],
    pub outline: [f32; 4],
    pub mesh_index: u32,
    pub is_skinned: u32,
    _pad0: u32,
    pub bone_transform_offset: u32,
}

impl EntityGPU {
    pub fn create(
        transform_matrix: Mat4,
        outline: [f32; 4],
        mesh_index: u32,
        is_skinned: bool,
        bone_transform_offset: u32,
        previous_transform_matrix: Mat4,
    ) -> Self {
        Self {
            transform_matrix: transform_matrix.to_cols_array_2d(),
            previous_transform_matrix: previous_transform_matrix.to_cols_array_2d(),
            outline,
            mesh_index,
            is_skinned: is_skinned as u32,
            _pad0: 0,
            bone_transform_offset,
        }
    }
}
