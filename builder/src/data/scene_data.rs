use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct SceneData {
    pub name: String,

    pub placeholders: Vec<EntityPlaceholderData>,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct EntityPlaceholderData {
    pub name: String,

    pub transform: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],

    pub mesh_asset_key: String,

    pub physical_body_type: BodyTypeData,
    pub physical_body_asset_key: String,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub enum BodyTypeData {
    Static,
    Kinematic,
    Dynamic,
}
