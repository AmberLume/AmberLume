use crate::data::common::primitive_data::PrimitiveData;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct MeshData {
    pub name: String,
    pub primitives: Vec<PrimitiveData>,
    pub bounds: [f32; 6],
}
