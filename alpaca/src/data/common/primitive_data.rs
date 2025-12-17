use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct PrimitiveData {
    pub indices: Vec<u32>,
    pub vertices: Vec<[f32; 3]>,
}
