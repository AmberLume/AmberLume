use ash::vk::Format;

pub const MAX_HIZ_MIPS: usize = 16;

#[derive(Copy, Clone)]
pub struct AmberLumeLimits {
    pub frames_in_flight: u32,
    pub resource_limits: ResourceLimits,
    pub shadow_map_limits: ShadowMapParams,
    pub hiz_limits: HiZParams,
    pub physics_limits: PhysicsLimits,
    pub profiler_limits: ProfilerLimits,
}

#[derive(Copy, Clone)]
pub struct ProfilerLimits {
    pub max_gpu_zones: u32,
}

#[derive(Copy, Clone)]
pub struct PhysicsLimits {
    pub fixed_delta_time: f32,
}

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
    pub max_sorted_draw_calls: u32,

    pub max_render_views: u32,

    pub max_texture_descriptors: u32,
    pub max_shadow_array_descriptors: u32,
    pub max_storage_image_descriptors: u32,
    pub max_graph_texture_descriptors: u32,
}

#[derive(Copy, Clone)]
pub struct ShadowMapParams {
    pub cascade_count: u32,
    pub max_distance: f32,

    pub resolution: u32,
    pub translucent_resolution: u32,
    pub format: ShadowMapFormat,

    pub bias: f32,
    pub normal_bias: f32,
    pub pcf_sample_count: u32,
    pub cascade_blend_range: f32,

    pub split_lambda: f32,
    pub shadow_caster_extension: f32,

    pub z_far_sample_stride: u32,
}

#[derive(Copy, Clone)]
pub enum ShadowMapFormat {
    D16,
    D32,
}

impl ShadowMapFormat {
    pub fn vulkan(&self) -> Format {
        match self {
            ShadowMapFormat::D16 => Format::D16_UNORM,
            ShadowMapFormat::D32 => Format::D32_SFLOAT,
        }
    }
}

#[derive(Copy, Clone)]
pub struct HiZParams {
    pub format: HiZFormat,
}

#[derive(Copy, Clone)]
pub enum HiZFormat {
    Rg16,
    Rg32,
}

impl HiZFormat {
    pub fn vulkan(&self) -> Format {
        match self {
            HiZFormat::Rg16 => Format::R16G16_SFLOAT,
            HiZFormat::Rg32 => Format::R32G32_SFLOAT,
        }
    }
}
