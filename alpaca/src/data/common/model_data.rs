use crate::data::common::mesh_data::MeshData;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct ModelData {
    pub name: String,
    pub meshes: Vec<MeshData>,
    pub bounds: [f32; 6],
}
