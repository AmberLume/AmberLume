use glam::{Vec2, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub _pad0: f32,
    pub normal: Vec3,
    pub _pad1: f32,
    pub uv: Vec2,
    pub _pad2: [f32; 2],
}

impl Vertex {
    pub fn create(position: Vec3, normal: Vec3, uv: Vec2) -> Self {
        Self {
            position,
            _pad0: 0.0,
            normal,
            _pad1: 0.0,
            uv,
            _pad2: [0.0; 2],
        }
    }
}
