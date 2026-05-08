use std::ops::Range;

pub struct AmberLumeLimits {
    pub frames_in_flight: u32,
    pub resource_limits: ResourceLimits,
    pub shadow_map_limits: ShadowMapParams,
}

pub struct ResourceLimits {
    pub max_entities: u32,

    pub max_frame_heap_size: u32,
    
    pub max_staging_size: u32,
    
    pub max_indices: u32,
    pub max_vertices: u32,

    pub max_meshes: u32,
    pub max_submeshes: u32,
    pub max_materials: u32,

    pub max_skeletons: u32,
    pub max_skeleton_bones: u32,
    pub max_bones_per_skeleton: u32,

    pub max_animations: u32,
    pub max_animation_frames: u32,
    
    pub max_skinning_instances: u32,
    pub max_bone_transforms: u32,

    pub max_draw_calls: u32,

    pub max_render_views: u32,

    pub max_texture_descriptors: u32,
    pub max_texture_array_descriptors: u32,
    pub max_shadow_descriptors: u32,
    pub max_shadow_array_descriptors: u32,
}

pub struct ShadowMapParams {
    pub global_cascades: Vec<Range<f32>>,

    pub resolution: u32,
    
    pub format: ShadowMapFormat,
    
    pub bias: f32,
    pub pcf_count: i32,
}

pub enum ShadowMapFormat {
    D16,
    D32,
}
