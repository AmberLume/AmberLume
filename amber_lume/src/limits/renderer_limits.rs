#[derive(Copy, Clone)]
pub struct RendererLimits {
    pub max_indices: u32,
    pub max_vertices: u32,
    
    pub max_submeshes: u32,
    pub max_materials: u32,
    pub max_models: u32,
    
    pub max_draw_calls: u32,
    pub max_entities: u32,
    
    pub max_textures: u32,
    pub max_views: u32,
    
    pub max_staging_size: u32,
}
