use crate::resources::common::resource_backend::ResourceKey;
use crate::resources::pipeline_layout::pipeline_layout_config::PipelineLayoutConfig;
use crate::resources::utils::hasher::hasher::Hasher;
use ash::vk::{
    BlendFactor, CompareOp, CullModeFlags, Format, FrontFace, PolygonMode, SampleCountFlags,
    ShaderStageFlags,
};

#[derive(Clone, Debug)]
pub struct PipelineConfig {
    pub stages: Vec<PipelineStageConfig>,

    pub color_formats: Vec<Format>,
    pub depth_format: Option<Format>,

    pub cull_mode: CullModeFlags,
    pub polygon_mode: PolygonMode,
    pub front_face: FrontFace,

    pub depth_test: bool,
    pub depth_write: bool,
    pub depth_compare_op: CompareOp,

    pub msaa_samples: SampleCountFlags,

    pub blend: bool,
    pub src_color_blend: BlendFactor,
    pub dst_color_blend: BlendFactor,

    pub pipeline_layout_config: PipelineLayoutConfig,
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

        hasher.hash_bool(self.depth_test);
        hasher.hash_bool(self.depth_write);
        hasher.hash_i32(self.depth_compare_op.as_raw());

        hasher.hash_u32(self.msaa_samples.as_raw());

        hasher.hash_bool(self.blend);
        hasher.hash_i32(self.src_color_blend.as_raw());
        hasher.hash_i32(self.dst_color_blend.as_raw());

        hasher.hash_resource_key(&self.pipeline_layout_config.hash());

        hasher.finalize()
    }
}
