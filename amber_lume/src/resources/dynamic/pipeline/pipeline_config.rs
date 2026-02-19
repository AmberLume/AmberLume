use crate::resources::utils::hasher::hasher::Hasher;
use ash::vk::{BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, Format, FrontFace, PolygonMode, PrimitiveTopology, SampleCountFlags, ShaderStageFlags};
use crate::resources::dynamic::resource_backend::ResourceKey;

#[derive(Clone, Debug)]
pub struct PipelineConfig {
    pub label: String,

    pub stages: Vec<PipelineStageConfig>,

    pub color_formats: Vec<Format>,
    pub depth_format: Option<Format>,

    pub cull_mode: CullModeFlags,
    pub polygon_mode: PolygonMode,
    pub front_face: FrontFace,
    pub primitive_topology: PrimitiveTopology,

    pub depth_test: bool,
    pub depth_write: bool,
    pub depth_compare_op: CompareOp,

    pub msaa_samples: SampleCountFlags,

    pub blend_enabled: bool,
    pub color_blend: Option<BlendConfig>,
    pub alpha_blend: Option<BlendConfig>,
    pub color_write_mask: ColorComponentFlags,
}

#[derive(Clone, Debug)]
pub struct BlendConfig {
    pub blend_op: BlendOp,
    pub src_blend: BlendFactor,
    pub dst_blend: BlendFactor,
}

#[derive(Clone, Debug)]
pub struct PipelineStageConfig {
    pub shader_name: String,
    pub fn_name: String,
    pub stage: ShaderStageFlags,
}

impl PipelineConfig {
    pub fn hash(&self) -> ResourceKey {
        let mut hasher = Hasher::new();

        hasher.hash_string(&self.label);

        for stage in &self.stages {
            hasher.hash_string(&stage.shader_name);
            hasher.hash_u32(stage.stage.as_raw());
            hasher.hash_string(&stage.fn_name);
        }

        hasher.hash_u32(self.color_formats.len() as u32);
        for color_format in &self.color_formats {
            hasher.hash_i32(color_format.as_raw())
        }
        hasher.hash_i32(self.depth_format.map(|f| f.as_raw()).unwrap_or(0));

        hasher.hash_u32(self.cull_mode.as_raw());
        hasher.hash_i32(self.polygon_mode.as_raw());
        hasher.hash_i32(self.front_face.as_raw());
        hasher.hash_i32(self.primitive_topology.as_raw());

        hasher.hash_bool(self.depth_test);
        hasher.hash_bool(self.depth_write);
        hasher.hash_i32(self.depth_compare_op.as_raw());

        hasher.hash_u32(self.msaa_samples.as_raw());

        hasher.hash_bool(self.blend_enabled);

        if let Some(color_blend) = &self.color_blend {
            hasher.hash_i32(color_blend.blend_op.as_raw());
            hasher.hash_i32(color_blend.src_blend.as_raw());
            hasher.hash_i32(color_blend.dst_blend.as_raw());
        } else {
            hasher.hash_bool(false);
        }
        if let Some(alpha_blend) = &self.alpha_blend {
            hasher.hash_i32(alpha_blend.blend_op.as_raw());
            hasher.hash_i32(alpha_blend.src_blend.as_raw());
            hasher.hash_i32(alpha_blend.dst_blend.as_raw());
        } else {
            hasher.hash_bool(false);
        }
        hasher.hash_u32(self.color_write_mask.as_raw());

        hasher.finalize()
    }
}
