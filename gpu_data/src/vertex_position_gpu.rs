use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct VertexPositionGPU {
    pub position: [f32; 3],
}

impl VertexPositionGPU {
    pub fn new(position: [f32; 3]) -> Self {
        Self { position }
    }
}
