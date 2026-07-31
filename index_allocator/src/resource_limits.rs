#[derive(Copy, Clone)]
pub struct ResourceLimits {
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
    pub max_transparent_draw_calls: u32,
    pub max_sorted_draw_calls: u32,

    pub max_render_views: u32,

    pub max_texture_descriptors: u32,
    pub max_shadow_array_descriptors: u32,
    pub max_storage_image_descriptors: u32,
    pub max_graph_texture_descriptors: u32,
}
