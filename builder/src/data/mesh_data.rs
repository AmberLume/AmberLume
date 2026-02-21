use rkyv::{Archive, Deserialize, Serialize};
use crate::data::submesh_data::SubmeshData;

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct MeshData {
    pub name: String,
    pub submeshes: Vec<SubmeshData>,
    pub bounds: [f32; 6],
}
