use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct MaterialData {
    pub base_color: [f32; 4],

    pub base_texture_id: Option<String>,
}
