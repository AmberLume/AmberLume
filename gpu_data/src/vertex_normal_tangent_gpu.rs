use bytemuck::{Pod, Zeroable};
use math::{Vec3, Vec4};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct VertexNormalTangentGPU {
    pub normal: Vec3,
    pub tangent: Vec4,
}

impl VertexNormalTangentGPU {
    pub fn new(normal: Vec3, tangent: Vec4) -> Self {
        Self { normal, tangent }
    }
}
