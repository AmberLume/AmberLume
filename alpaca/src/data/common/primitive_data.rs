use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct PrimitiveData {
    pub material_id: String,

    pub indices: Vec<u32>,
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
}
