use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct VertexNormalTangentGPU {
    pub normal: [f32; 3],
    pub tangent: [f32; 4],
}

impl VertexNormalTangentGPU {
    pub fn new(normal: [f32; 3], tangent: [f32; 4]) -> Self {
        Self { normal, tangent }
    }
}
