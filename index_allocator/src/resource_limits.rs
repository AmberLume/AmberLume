#[derive(Copy, Clone)]
pub struct ResourceLimits {
    pub upload_heap_block_size: u32,
    pub device_heap_block_size: u32,

    pub max_staging_size: u32,

    pub max_indices: u32,
    pub max_vertices: u32,
    pub max_vertex_attributes: u32,
    pub max_vertex_skins: u32,
    pub max_mesh_bones: u32,

    pub max_meshes: u32,
    pub max_submeshes: u32,
    pub max_materials: u32,

    pub max_skeletons: u32,
    pub max_skeleton_bones: u32,

    pub max_animations: u32,
    pub max_animation_frames: u32,

    pub max_draw_calls: u32,
    pub max_transparent_draw_calls: u32,
    pub max_sorted_draw_calls: u32,

    pub max_texture_descriptors: u32,
    pub max_shadow_array_descriptors: u32,
    pub max_storage_image_descriptors: u32,
    pub max_graph_texture_descriptors: u32,
}
