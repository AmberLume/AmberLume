use crate::render::vulkan::device_context::DeviceContext;
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::common::resource_provider::{ResourceId, ResourceProvider};
use crate::resources::pipeline::pipeline_config::PipelineConfig;
use crate::resources::pipeline_layout::pipeline_layout_backend::PipelineLayoutBackend;
use crate::resources::shader::shader_backend::ShaderBackend;
use crate::resources::shader::shader_config::ShaderConfig;
use anyhow::Result;
use ash::vk::{
    BlendOp, ColorComponentFlags, DynamicState, Format, GraphicsPipelineCreateInfo, Pipeline,
    PipelineCache, PipelineColorBlendAttachmentState, PipelineDepthStencilStateCreateInfo,
    PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineMultisampleStateCreateInfo,
    PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo,
    PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PrimitiveTopology,
    ShaderModule, ShaderStageFlags,
};
use ash::{Device, vk};
use std::array::from_ref;
use std::ffi::CString;
use std::sync::Arc;
use tracing::info;
use vk::{
    PipelineColorBlendStateCreateInfo, PipelineDynamicStateCreateInfo, PipelineRenderingCreateInfo,
};

pub struct PipelineBackend {
    device: Device,

    shader_provider: Arc<ResourceProvider<ShaderBackend>>,
    pipeline_layout_provider: Arc<ResourceProvider<PipelineLayoutBackend>>,

    pipeline_cache: PipelineCache,
}

impl PipelineBackend {
    pub fn new(
        device_context: &DeviceContext,
        shader_provider: Arc<ResourceProvider<ShaderBackend>>,
        pipeline_layout_provider: Arc<ResourceProvider<PipelineLayoutBackend>>,
        pipeline_cache: PipelineCache,
    ) -> Self {
        Self {
            device: device_context.device.clone(),

            shader_provider,
            pipeline_layout_provider,

            pipeline_cache,
        }
    }
}

pub struct PipelineDependencies {
    pub shader_stages: Vec<PipelineShaderStage>,
    pub pipeline_layout: PipelineLayout,
}

pub struct PipelineShaderStage {
    pub fn_name: CString,
    pub stage: ShaderStageFlags,
    pub shader_module: ShaderModule,
}

impl ResourceBackend for PipelineBackend {
    type Config = PipelineConfig;
    type Dependencies = PipelineDependencies;
    type Output = Pipeline;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn collect_dependencies(&self, config: &Self::Config) -> Self::Dependencies {
        let mut shader_stages = Vec::with_capacity(config.stages.len());

        for stage in &config.stages {
            let config = ShaderConfig {
                name: stage.shader_name.clone(),
            };

            self.shader_provider.touch(&config);
            let shader_module_id = self.shader_provider.get_id(&config);
            let shader_module = self
                .shader_provider
                .get_ready(&shader_module_id, true)
                .unwrap();

            let shader_stage = PipelineShaderStage {
                fn_name: CString::new(stage.fn_name.clone()).unwrap(),
                stage: stage.stage,
                shader_module: *shader_module,
            };

            shader_stages.push(shader_stage);
        }

        let pipeline_layout = self
            .pipeline_layout_provider
            .get_now(&config.pipeline_layout_config)
            .unwrap();

        Self::Dependencies {
            shader_stages,
            pipeline_layout: *pipeline_layout,
        }
    }

    fn create(
        &self,
        _id: &ResourceId,
        config: Self::Config,
        dependencies: Self::Dependencies,
    ) -> Result<Self::Output> {
        let mut shader_stages = Vec::with_capacity(dependencies.shader_stages.len());

        for stage in &dependencies.shader_stages {
            let stage_create_info = PipelineShaderStageCreateInfo::default()
                .name(&stage.fn_name)
                .stage(stage.stage)
                .module(stage.shader_module);

            shader_stages.push(stage_create_info);
        }

        let vertex_input_info = PipelineVertexInputStateCreateInfo::default();

        let input_assembly_info = PipelineInputAssemblyStateCreateInfo::default()
            .primitive_restart_enable(false)
            .topology(PrimitiveTopology::TRIANGLE_LIST);

        let viewport_state = PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_info = PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(config.polygon_mode)
            .cull_mode(config.cull_mode)
            .front_face(config.front_face)
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .depth_bias_enable(false)
            .line_width(1.0);

        let multisample_info = PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(config.msaa_samples);

        let depth_stencil_info = PipelineDepthStencilStateCreateInfo::default()
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .depth_test_enable(config.depth_test)
            .depth_write_enable(config.depth_write)
            .depth_compare_op(config.depth_compare_op);

        let color_blend_attachment = if config.blend {
            PipelineColorBlendAttachmentState::default()
                .blend_enable(false)
                .src_color_blend_factor(config.src_color_blend)
                .dst_color_blend_factor(config.dst_color_blend)
                .color_blend_op(BlendOp::ADD)
                .color_write_mask(ColorComponentFlags::RGBA)
        } else {
            PipelineColorBlendAttachmentState::default()
                .blend_enable(false)
                .color_write_mask(ColorComponentFlags::RGBA)
        };

        let color_blend_info = PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(from_ref(&color_blend_attachment));

        let dynamic_states = [DynamicState::VIEWPORT, DynamicState::SCISSOR];
        let dynamic_state_info =
            PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let mut rendering_info = PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&config.color_formats)
            .depth_attachment_format(config.depth_format.unwrap_or(Format::UNDEFINED))
            .stencil_attachment_format(Format::UNDEFINED);

        let pipeline_info = GraphicsPipelineCreateInfo::default()
            .push_next(&mut rendering_info)
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_info)
            .depth_stencil_state(&depth_stencil_info)
            .color_blend_state(&color_blend_info)
            .dynamic_state(&dynamic_state_info)
            .layout(dependencies.pipeline_layout);

        let pipeline = unsafe {
            self.device
                .create_graphics_pipelines(self.pipeline_cache, &[pipeline_info], None)
                .map(|pipelines| pipelines[0])
                .unwrap()
        };

        Ok(pipeline)
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        unsafe { self.device.destroy_pipeline(resource, None) }

        info!("Pipeline destroyed");

        Ok(())
    }
}
