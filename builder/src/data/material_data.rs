use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct MaterialData {
    pub base_color_factor: [f32; 4],
    pub roughness_factor: f32,
    pub metallic_factor: f32,

    pub base_texture_id: Option<String>,
    pub normal_texture_id: Option<String>,
    pub occlusion_roughness_metalic_texture_id: Option<String>,
}
