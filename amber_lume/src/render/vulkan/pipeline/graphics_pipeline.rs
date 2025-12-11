use crate::resources::pipeline::pipeline_config::{PipelineConfig, PipelineStageConfig};
use crate::resources::pipeline_layout::pipeline_layout_config::PipelineLayoutConfig;
use crate::resources::resource_hub::ResourceHub;
use anyhow::Result;
use ash::vk;
use ash::vk::{
    BlendFactor, CompareOp, CullModeFlags, Format, FrontFace, PolygonMode, SampleCountFlags,
};
use std::sync::Arc;
use vk::{Pipeline, ShaderStageFlags};

pub struct GraphicsPipeline {
    pipeline: Arc<Pipeline>,
}

impl GraphicsPipeline {
    pub fn create(resource_hub: Arc<ResourceHub>) -> Result<Self> {
        let pipeline_stages = vec![
            PipelineStageConfig {
                shader_name: String::from("depth.frag.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::FRAGMENT,
            },
            PipelineStageConfig {
                shader_name: String::from("depth.vert.spv"),
                fn_name: String::from("main"),
                stage: ShaderStageFlags::VERTEX,
            },
        ];

        let pipeline_layout_config = PipelineLayoutConfig {
            descriptor_set_layout_configs: vec![],
            push_constant_ranges: vec![],
        };

        let pipeline_config = PipelineConfig {
            stages: pipeline_stages,

            color_formats: vec![],
            depth_format: Some(Format::D32_SFLOAT),

            cull_mode: CullModeFlags::BACK,
            polygon_mode: PolygonMode::FILL,
            front_face: FrontFace::COUNTER_CLOCKWISE,

            depth_test: true,
            depth_write: true,
            depth_compare_op: CompareOp::LESS,

            msaa_samples: SampleCountFlags::TYPE_1,

            blend: false,
            src_color_blend: BlendFactor::ONE,
            dst_color_blend: BlendFactor::ZERO,

            pipeline_layout_config,
        };

        let pipeline_provider = resource_hub.get_pipeline_provider();

        let id = pipeline_provider.get_id(&pipeline_config);
        let pipeline = pipeline_provider.get_now(&pipeline_config).unwrap();

        println!("Pipeline config: {}", id);
        println!("Pipeline id: {}", id);
        println!("Pipeline: {:#?}", &pipeline);

        Ok(Self { pipeline })
    }
}
