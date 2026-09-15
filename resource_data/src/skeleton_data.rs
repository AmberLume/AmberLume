use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct SkeletonData {
    pub name: String,

    pub bones: Vec<BoneData>,
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BoneData {
    pub name: String,
    pub parent_index: i32,

    pub rest_translation: [f32; 3],
    pub rest_rotation: [f32; 4],
    pub rest_scale: [f32; 3],
}
