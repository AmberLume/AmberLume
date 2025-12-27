use alpaca::data::common::primitive_data::PrimitiveData;
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

    pub fn from(primitive_data: &PrimitiveData, index: usize) -> Self {
        let vertex = &primitive_data.vertices[index];
        let normal = &primitive_data.normals[index];

        Self::create(
            Vec3::new(vertex[0], vertex[1], vertex[2]),
            Vec3::new(normal[0], normal[1], normal[2]),
            Vec2::ZERO,
        )
    }
}
