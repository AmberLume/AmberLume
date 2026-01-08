use rkyv::{Archive, Deserialize, Serialize};
use crate::data::primitive_data::PrimitiveData;

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct MeshData {
    pub name: String,
    pub primitives: Vec<PrimitiveData>,
    pub bounds: [f32; 6],
}
