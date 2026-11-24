use bytemuck::{Pod, Zeroable};
use math::{Vec3, Vec4};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct PhysicsDebugVertexGPU {
    pub point: Vec3,

    _pad0: u32,

    pub color: Vec4,
}

impl PhysicsDebugVertexGPU {
    pub fn new(point: Vec3, color: Vec4) -> Self {
        Self {
            point,

            _pad0: 0,

            color,
        }
    }
}
