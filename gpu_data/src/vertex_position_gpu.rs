use bytemuck::{Pod, Zeroable};
use math::Vec3;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct VertexPositionGPU {
    pub position: Vec3,
}

impl VertexPositionGPU {
    pub fn new(position: Vec3) -> Self {
        Self { position }
    }
}
