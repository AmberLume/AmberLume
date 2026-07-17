use glam::{Quat, Vec3};
use crate::data::physical_body_data::{ArchivedColliderData, ArchivedColliderShape, ArchivedPhysicalBodyData};
use physics::BodyType;

#[derive(Debug)]
pub struct PhysicalBodyBlueprint {
    pub body_type: BodyType,

    pub scale: Vec3,

    pub physical_body_asset_key: String,
}

pub struct PhysicalBodyData {
    pub name: String,

    pub colliders: Vec<ColliderData>,
}

impl PhysicalBodyData {
    pub fn from_rkyv(data: &ArchivedPhysicalBodyData) -> Self {
        Self {
            name: data.name.as_str().to_string(),

            colliders: data.colliders.iter().map(|collider| ColliderData::from_rkyv(collider)).collect(),
        }
    }
}

pub struct ColliderData {
    pub name: String,

    pub shape: ColliderShape,

    pub density: f32,
    pub friction: f32,
    pub restitution: f32,

    pub translation: Vec3,
    pub rotation: Quat,
}

impl ColliderData {
    pub fn from_rkyv(data: &ArchivedColliderData) -> Self {
        Self {
            name: data.collider_name.as_str().to_string(),
            shape: ColliderShape::from_rkyv(&data.collider_shape),

            density: data.density.into(),
            friction: data.friction.into(),
            restitution: data.restitution.into(),

            translation: Vec3::from_array(data.translation.map(|v| v.into())),
            rotation: Quat::from_array(data.rotation.map(|v| v.into())),
        }
    }
}

pub enum ColliderShape {
    Box {
        size: [f32; 3],
    },
    ConvexHull {
        vertices: Vec<[f32; 3]>,
    },
}

impl ColliderShape {
    pub fn from_rkyv(data: &ArchivedColliderShape) -> Self {
        match data {
            ArchivedColliderShape::Box { size } => Self::Box {
                size: size.map(|v| v.into())
            },
            ArchivedColliderShape::ConvexHull { vertices } => Self::ConvexHull {
                vertices: vertices.iter().map(|v| {
                    v.map(|v| v.into())
                }).collect()
            }
        }
    }
}
