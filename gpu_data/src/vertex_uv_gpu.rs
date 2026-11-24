use bytemuck::{Pod, Zeroable};
use math::Vec2;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct VertexUvGPU {
    pub uv: Vec2,
}

impl VertexUvGPU {
    pub fn new(uv: Vec2) -> Self {
        Self { uv }
    }
}
