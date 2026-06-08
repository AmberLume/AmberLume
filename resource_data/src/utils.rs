use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct AssetLink {
    pub asset_name: String,
    pub file_path: String,
}
