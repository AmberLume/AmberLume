use crate::resource_key::ResourceKey;
use crate::submesh_data::SubmeshData;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct MeshData {
    pub name: String,

    pub submeshes: Vec<SubmeshData>,

    pub skeleton: Option<ResourceKey>,

    pub bounds: [f32; 6],
}
