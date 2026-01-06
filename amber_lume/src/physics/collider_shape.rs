use glam::Vec3;
use alpaca::data::common::scene_data::ColliderShapeData;

#[derive(Debug)]
pub enum ColliderShape {
    Box {
        size: Vec3,
    },
}

impl ColliderShape {
    pub fn from_data(collider_shape_data: &ColliderShapeData) -> Self {
        match collider_shape_data {
            ColliderShapeData::Box { size } => ColliderShape::Box { 
                size: Vec3::new(size[0], size[1], size[2]),
            },
        }
    }
}
