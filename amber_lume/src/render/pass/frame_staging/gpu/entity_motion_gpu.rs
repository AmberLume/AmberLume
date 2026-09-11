use bytemuck::{Pod, Zeroable};
use glam::Mat4;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct EntityMotionGPU {
    pub previous_transform_matrix: [[f32; 4]; 4],
}

impl EntityMotionGPU {
    pub fn create(previous_transform_matrix: Mat4) -> Self {
        Self {
            previous_transform_matrix: previous_transform_matrix.to_cols_array_2d(),
        }
    }
}
