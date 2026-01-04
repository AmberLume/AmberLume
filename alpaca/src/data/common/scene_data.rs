use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct SceneData {
    pub name: String,

    pub nodes: Vec<SceneNodeData>,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct SceneNodeData {
    pub name: String,

    pub transform: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],

    pub asset_key: String,

    pub colliders: Vec<SceneNodeCollider>,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct SceneNodeCollider {
    pub collider_name: String,

    pub collider_type: ColliderType,
    pub collider_shape: ColliderShape,

    pub rotation: [f32; 4],
    pub position: [f32; 3],
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub enum ColliderType {
    Static,
    Kinematic,
    Dynamic,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub enum ColliderShape {
    Box {
        size: [f32; 3],
    }
}
