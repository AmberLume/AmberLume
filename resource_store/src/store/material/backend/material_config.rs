use std::sync::Arc;
use resource_data::alpha_mode::AlphaMode;
use resource_residency::ResRef;

#[derive(Clone)]
pub struct MaterialConfig {
    pub base_color_factor: [f32; 4],
    pub roughness_factor: f32,
    pub metallic_factor: f32,

    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,

    pub color_image: Arc<ResRef>,
    pub normal_image: Arc<ResRef>,
    pub orm_image: Arc<ResRef>,
}
