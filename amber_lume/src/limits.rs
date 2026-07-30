use ash::vk::Format;

pub use gpu::ResourceLimits;

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
pub struct ShadowMapParams {
    pub cascade_count: u32,
    pub max_distance: f32,

    pub resolution: u32,
    pub format: ShadowMapFormat,

    pub bias: f32,
    pub normal_bias: f32,
    pub pcf_sample_count: u32,
    pub pcf_world_radius: f32,
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
