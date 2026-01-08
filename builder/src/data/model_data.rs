use rkyv::{Archive, Deserialize, Serialize};
use crate::data::mesh_data::MeshData;

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct ModelData {
    pub meshes: Vec<MeshData>,
    pub bounds: [f32; 6],
}
