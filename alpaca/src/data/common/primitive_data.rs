use crate::data::common::material_data::MaterialData;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct PrimitiveData {
    pub material_data: MaterialData,

    pub indices: Vec<u32>,
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
}
