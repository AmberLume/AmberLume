use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct MaterialData {
    pub base_texture_key: Option<String>,
}
