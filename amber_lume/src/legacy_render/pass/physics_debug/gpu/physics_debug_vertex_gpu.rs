use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct PhysicsDebugVertexGPU {
    pub point: [f32; 3],

    _pad0: u32,

    pub color: [f32; 4],
}

impl PhysicsDebugVertexGPU {
    pub fn new(point: [f32; 3], color: [f32; 4]) -> Self {
        Self {
            point,

            _pad0: 0,

            color,
        }
    }
}
