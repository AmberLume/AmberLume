use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct VertexUvGPU {
    pub uv: [f32; 2],
}

impl VertexUvGPU {
    pub fn new(uv: [f32; 2]) -> Self {
        Self { uv }
    }
}
