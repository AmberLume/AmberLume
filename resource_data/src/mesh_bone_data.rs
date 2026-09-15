use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct MeshBoneData {
    pub inverse_bind_matrix: [[f32; 4]; 4],
    pub bounds: [f32; 6],
}
