use glam::Vec4;
use builder::data::scene_data::BodyColliderShapeData;

#[derive(Debug, Copy, Clone)]
pub struct ColliderShape {
    pub shape_type: ShapeType,
    pub half_extents: Vec4,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum ShapeType {
    Box = 0,
}

impl ColliderShape {
    pub fn from_data(body_collider_shape_data: &BodyColliderShapeData) -> Self {
        let (shape_type, half_extents) = match body_collider_shape_data {
            BodyColliderShapeData::Box { size } => {
                (ShapeType::Box, Vec4::new(size[0] / 2.0, size[1] / 2.0, size[2] / 2.0, 0.0))
            },
        };

        Self {
            shape_type,

            half_extents,
        }
    }
}
