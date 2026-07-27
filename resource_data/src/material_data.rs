use rkyv::{Archive, Deserialize, Serialize};
use crate::alpha_mode::AlphaMode;
use crate::resource_key::ResourceKey;

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct MaterialData {
    pub base_color_factor: [f32; 4],
    pub roughness_factor: f32,
    pub metallic_factor: f32,

    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,

    pub base_texture_id: Option<ResourceKey>,
    pub normal_texture_id: Option<ResourceKey>,
    pub occlusion_roughness_metallic_texture_id: Option<ResourceKey>,
}
