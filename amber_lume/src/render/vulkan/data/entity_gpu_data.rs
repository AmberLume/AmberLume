use glam::Mat4;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct EntityGpuData {
    pub transform_matrix: Mat4,
}

impl EntityGpuData {
    pub fn create(transform_matrix: Mat4) -> Self {
        Self {
            transform_matrix,
        }
    }
}
