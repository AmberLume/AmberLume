use rkyv::{Archive, Deserialize, Serialize};
use crate::resource_key::ResourceKey;

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

    pub mesh: Option<ResourceKey>,

    pub physical_body_type: BodyTypeData,
    pub physical_body: ResourceKey,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq, serde::Deserialize)]
pub enum BodyTypeData {
    Static,
    Kinematic,
    Dynamic,
}
