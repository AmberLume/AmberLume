use std::hash::{Hash, Hasher};
use ash::vk::{BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, Format, FrontFace, PolygonMode, PrimitiveTopology, SampleCountFlags, ShaderStageFlags};
use crate::data::resource_handle::ShaderResource;

#[derive(Clone, Debug)]
pub struct PipelineConfig {
    pub label: String,

    pub stages: Vec<PipelineStageConfig>,

    pub color_formats: Vec<Format>,
    pub depth_format: Option<Format>,
    pub view_mask: u32,

    pub cull_mode: CullModeFlags,
    pub polygon_mode: PolygonMode,
    pub front_face: FrontFace,
    pub primitive_topology: PrimitiveTopology,

    pub depth_bias_enable: bool,
    pub depth_bias_constant_factor: f32,
    pub depth_bias_slope_factor: f32,

    pub depth_test: bool,
    pub depth_write: bool,
    pub depth_compare_op: CompareOp,

    pub msaa_samples: SampleCountFlags,

    pub blend_enabled: bool,
    pub color_blend: Option<BlendConfig>,
    pub alpha_blend: Option<BlendConfig>,
    pub color_write_mask: ColorComponentFlags,
}

impl Hash for PipelineConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self {
            label,
            
            stages,

            color_formats,
            depth_format,
            view_mask,

            cull_mode,
            polygon_mode,
            front_face,
            primitive_topology,

            depth_bias_enable,
            depth_bias_constant_factor,
            depth_bias_slope_factor,

            depth_test,
            depth_write,
            depth_compare_op,

            msaa_samples,

            blend_enabled,
            color_blend,
            alpha_blend,
            color_write_mask,
        } = self;

        label.hash(state);

        stages.hash(state);

        color_formats.hash(state);
        depth_format.hash(state);
        view_mask.hash(state);

        cull_mode.hash(state);
        polygon_mode.hash(state);
        front_face.hash(state);
        primitive_topology.hash(state);
        
        depth_bias_enable.hash(state);
        depth_bias_constant_factor.to_bits().hash(state);
        depth_bias_slope_factor.to_bits().hash(state);
        
        depth_test.hash(state);
        depth_write.hash(state);
        depth_compare_op.hash(state);
        
        msaa_samples.hash(state);
        
        blend_enabled.hash(state);
        color_blend.hash(state);
        alpha_blend.hash(state);
        color_write_mask.hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct BlendConfig {
    pub blend_op: BlendOp,
    pub src_blend: BlendFactor,
    pub dst_blend: BlendFactor,
}

impl Hash for BlendConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self { 
            blend_op, 
            src_blend, 
            dst_blend,
        } = self;
        
        blend_op.hash(state);
        src_blend.hash(state);
        dst_blend.hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct PipelineStageConfig {
    pub shader_name: ShaderResource,
    pub fn_name: String,
    pub stage: ShaderStageFlags,
}

impl Hash for PipelineStageConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self { 
            shader_name, 
            fn_name, 
            stage,
        } = self;
        
        shader_name.hash(state);
        fn_name.hash(state);
        stage.hash(state);
    }
}

impl PipelineStageConfig {
    pub fn vertex(shader_name: ShaderResource) -> Self {
        Self {
            shader_name,
            fn_name: String::from("main"),
            stage: ShaderStageFlags::VERTEX,
        }
    }

    pub fn fragment(shader_name: ShaderResource) -> Self {
        Self {
            shader_name,
            fn_name: String::from("main"),
            stage: ShaderStageFlags::FRAGMENT,
        }
    }
}

impl BlendConfig {
    pub fn replace() -> Self {
        Self {
            blend_op: BlendOp::ADD,
            src_blend: BlendFactor::ONE,
            dst_blend: BlendFactor::ZERO,
        }
    }

    pub fn additive() -> Self {
        Self {
            blend_op: BlendOp::ADD,
            src_blend: BlendFactor::ONE,
            dst_blend: BlendFactor::ONE,
        }
    }

    pub fn alpha() -> Self {
        Self {
            blend_op: BlendOp::ADD,
            src_blend: BlendFactor::SRC_ALPHA,
            dst_blend: BlendFactor::ONE_MINUS_SRC_ALPHA,
        }
    }

    pub fn premultiplied_alpha() -> Self {
        Self {
            blend_op: BlendOp::ADD,
            src_blend: BlendFactor::ONE,
            dst_blend: BlendFactor::ONE_MINUS_SRC_ALPHA,
        }
    }
}

impl PipelineConfig {
    pub fn fullscreen() -> Self {
        Self {
            label: String::new(),

            stages: Vec::new(),

            color_formats: Vec::new(),
            depth_format: None,
            view_mask: 0,

            cull_mode: CullModeFlags::NONE,
            polygon_mode: PolygonMode::FILL,
            front_face: FrontFace::COUNTER_CLOCKWISE,
            primitive_topology: PrimitiveTopology::TRIANGLE_LIST,

            depth_bias_enable: false,
            depth_bias_constant_factor: 0.0,
            depth_bias_slope_factor: 0.0,

            depth_test: false,
            depth_write: false,
            depth_compare_op: CompareOp::ALWAYS,

            msaa_samples: SampleCountFlags::TYPE_1,

            blend_enabled: false,
            color_blend: Some(BlendConfig::replace()),
            alpha_blend: None,
            color_write_mask: ColorComponentFlags::RGBA,
        }
    }

    pub fn geometry() -> Self {
        Self {
            cull_mode: CullModeFlags::BACK,

            depth_test: true,
            depth_write: true,
            depth_compare_op: CompareOp::GREATER_OR_EQUAL,

            ..Self::fullscreen()
        }
    }
}
