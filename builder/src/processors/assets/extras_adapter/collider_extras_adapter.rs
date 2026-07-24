use anyhow::bail;
use anyhow::Result;
use gltf::Node;
use serde::Deserialize;
use serde_json::from_str;

#[derive(Deserialize, Debug, PartialEq)]
pub enum ColliderShapeType {
    Box,
    Capsule,
    ConvexHull,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ColliderExtras {
    pub collider_name: String,
    pub collider_shape: ColliderShapeType,

    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
}

impl ColliderExtras {
    pub fn adapt(node: &Node) -> Result<ColliderExtras> {
        let Some(extras) = node.extras() else {
            bail!("No extras");
        };

        match from_str::<ColliderExtras>(extras.get()) {
            Ok(extras) => Ok(extras),
            Err(error) => bail!("Invalid extras: {}", error),
        }
    }
}
