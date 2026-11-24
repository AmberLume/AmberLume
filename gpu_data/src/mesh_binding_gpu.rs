use bytemuck::{Pod, Zeroable};
use math::{Mat4, Vec4};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct MeshBindingGPU {
    pub inverse_bind_matrix: Mat4,

    pub bounds_min: Vec4,
    pub bounds_max: Vec4,
}

impl MeshBindingGPU {
    pub fn create(inverse_bind_matrix: Mat4, bounds: [f32; 6]) -> Self {
        Self {
            inverse_bind_matrix,

            bounds_min: Vec4::new(bounds[0], bounds[1], bounds[2], 0.0),
            bounds_max: Vec4::new(bounds[3], bounds[4], bounds[5], 0.0),
        }
    }
}
