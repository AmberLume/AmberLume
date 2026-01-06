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

    pub physical_body: PhysicalBodyData,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct PhysicalBodyData {
    pub body_type: BodyTypeData,

    pub colliders: Vec<SceneNodeColliderData>,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub enum BodyTypeData {
    Static,
    Kinematic,
    Dynamic,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct SceneNodeColliderData {
    pub collider_name: String,

    pub collider_shape: ColliderShapeData,

    pub rotation: [f32; 4],
    pub position: [f32; 3],
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub enum ColliderShapeData {
    Box {
        size: [f32; 3],
    }
}
