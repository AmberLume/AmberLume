use crate::resource_key::ResourceKey;
use crate::submesh_data::SubmeshData;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct MeshData {
    pub name: String,

    pub submeshes: Vec<SubmeshData>,

    pub skeleton: Option<ResourceKey>,
    pub inverse_bind_matrices: Vec<[[f32; 4]; 4]>,

    pub bounds: [f32; 6],
}
