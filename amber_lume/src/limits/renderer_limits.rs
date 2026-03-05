#[derive(Copy, Clone)]
pub struct RendererLimits {
    pub frames_in_flight: usize,
    pub buffer_limits: BufferLimits,
    pub render_resource_limits: RenderResourceLimits,
    pub image_resource_limits: ImageResourceLimits,
    pub shadow_map_limits: ShadowMapParams,
}

#[derive(Copy, Clone)]
pub struct BufferLimits {
    pub max_entities: u32,
    
    pub max_staging_size: u32,
}

#[derive(Copy, Clone)]
pub struct RenderResourceLimits {
    pub max_indices: u32,
    pub max_vertices: u32,

    pub max_submeshes: u32,
    pub max_materials: u32,
    pub max_models: u32,

    pub max_draw_calls: u32,

    pub max_render_views: u32,
}

#[derive(Copy, Clone)]
pub struct ImageResourceLimits {
    pub max_texture_descriptors: u32,
    pub max_texture_array_descriptors: u32,

    pub max_shadow_descriptors: u32,
    pub max_shadow_array_descriptors: u32,
}

#[derive(Copy, Clone)]
pub struct ShadowMapParams {
    pub global_cascade_count: u32,

    pub resolution: u32,
    
    pub format: ShadowMapFormat,
    
    pub bias: f32,
    pub pcf_count: i32,
}

#[derive(Copy, Clone)]
pub enum ShadowMapFormat {
    D16,
    D32,
}
