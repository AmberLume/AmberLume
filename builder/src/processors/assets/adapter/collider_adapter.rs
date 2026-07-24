use anyhow::Result;
use gltf::Node;
use resource_data::physical_body_data::ColliderShape;
use crate::processors::assets::extras_adapter::collider_extras_adapter::{ColliderExtras, ColliderShapeType};
use crate::processors::assets::extras_adapter::geometry_extras_adapter::Geometry;
use crate::processors::assets::extras_adapter::position_adapter::Position;

#[derive(Debug, PartialEq)]
pub struct Collider {
    pub name: String,

    pub shape: ColliderShape,

    pub friction: f32,
    pub restitution: f32,
    pub density: f32,

    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Collider {
    pub fn adapt(node: &Node, bin: Option<&[u8]>) -> Result<Collider> {
        let collider_extras = ColliderExtras::adapt(node)?;
        let position = Position::adapt(node)?;

        Ok(Self {
            name: collider_extras.collider_name,

            shape: match collider_extras.collider_shape {
                ColliderShapeType::Box => ColliderShape::Box,
                ColliderShapeType::Capsule => ColliderShape::Capsule,
                ColliderShapeType::ConvexHull => ColliderShape::ConvexHull {
                    vertices: Geometry::adapt(node, bin)?.positions,
                },
            },

            friction: collider_extras.friction,
            restitution: collider_extras.restitution,
            density: collider_extras.density,

            translation: position.translation,
            rotation: position.rotation,
            scale: position.scale,
        })
    }
}
