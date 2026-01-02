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
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct SceneNodeExtrasData {
    pub asset_path: String,
}

