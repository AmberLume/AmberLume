use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct SkeletonData {
    pub name: String,

    pub bones: Vec<BoneData>,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct BoneData {
    pub name: String,
    pub parent_index: i32,

    pub inverse_bind_matrix: [[f32; 4]; 4],
}
