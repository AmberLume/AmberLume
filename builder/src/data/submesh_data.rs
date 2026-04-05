use rkyv::{Archive, Deserialize, Serialize};
use crate::data::resource_key::ResourceKey;

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct SubmeshData {
    pub material: Option<ResourceKey>,

    pub indices: Vec<u32>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
    pub bone_indices: Vec<[u16; 4]>,
    pub bone_weights: Vec<[f32; 4]>,

    pub bounds: [f32; 6],
}
