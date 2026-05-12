use crate::render::utils::debug_utils::DebugUtils;
use crate::resources::store::providers::resource_backend::ResourceBackend;
use crate::resources::store::providers::resource_provider::ResourceId;
use crate::resources::alpaca_resource_reader::AlpacaResourceReader;
use anyhow::Result;
use ash::vk::{BlendOp, DynamicState, Format, GraphicsPipelineCreateInfo, Pipeline, PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo, PipelineDepthStencilStateCreateInfo, PipelineDynamicStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo, PipelineRenderingCreateInfo, PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, ShaderModule, ShaderModuleCreateInfo};
use bytemuck::cast_slice;
use std::array::from_ref;
use std::ffi::CString;
use std::sync::Arc;
use ash::Device;
use tracing::info;
use crate::resources::binding_layout::binding_layout::BindingLayout;
use crate::resources::binding_layout::pipeline_layout_registry::PipelineLayoutType;
use crate::resources::store::providers::pipeline::pipeline_config::PipelineConfig;

pub struct PipelineBackend {
    device: Device,
    debug_utils: Arc<DebugUtils>,

    alpaca_resource_reader: Arc<AlpacaResourceReader>,

    binding_layout: Arc<BindingLayout>,

    pipeline_cache: PipelineCache,
}

impl PipelineBackend {
    pub fn new(
        device: Device,
        debug_utils: Arc<DebugUtils>,
        pipeline_cache: PipelineCache,
        alpaca_resource_reader: Arc<AlpacaResourceReader>,
        binding_layout: Arc<BindingLayout>,
    ) -> Self {
        Self {
            device: device.clone(),
            debug_utils: debug_utils.clone(),

            alpaca_resource_reader,

            binding_layout,

            pipeline_cache,
        }
    }

    fn create_shader_module(&self, label: &str, spv: &[u32]) -> Result<ShaderModule> {
        let shader_module_create_info = ShaderModuleCreateInfo::default().code(spv);

        let shader_module = unsafe {
            self.device.create_shader_module(&shader_module_create_info, None)?
        };

        self.debug_utils
            .label(shader_module, &format!("shader_module_{}", label));

        Ok(shader_module)
    }
}

impl ResourceBackend for PipelineBackend {
    type Config = PipelineConfig;
    type Output = Pipeline;
    type Statistics = ();

    fn create(&self, _id: &ResourceId, config: Self::Config) -> Result<Self::Output> {
        let mut shader_modules = Vec::with_capacity(config.stages.len());
        let mut entry_points = Vec::with_capacity(config.stages.len());

        let mut prepare_shaders = || -> Result<()> {
            for stage_config in &config.stages {
                let spv = self
                    .alpaca_resource_reader
                    .get_resource(&stage_config.shader_name)?;

                let shader_module = self.create_shader_module(&stage_config.shader_name, cast_slice(spv))?;

                shader_modules.push(shader_module);
                entry_points.push(CString::new(stage_config.fn_name.clone())?);
            }

            Ok(())
        };

        if let Err(e) = prepare_shaders() {
            for module in shader_modules {
                unsafe { self.device.destroy_shader_module(module, None) };
            }
            return Err(e);
        }

        let shader_stages = config
            .stages
            .iter()
            .enumerate()
            .map(|(index, stage_config)| {
                PipelineShaderStageCreateInfo::default()
                    .name(entry_points[index].as_c_str())
                    .stage(stage_config.stage)
                    .module(shader_modules[index])
            })
            .collect::<Vec<_>>();

        let vertex_input_info = PipelineVertexInputStateCreateInfo::default();

        let input_assembly_info = PipelineInputAssemblyStateCreateInfo::default()
            .primitive_restart_enable(false)
            .topology(config.primitive_topology);

        let viewport_state = PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let mut rasterization_info = PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(config.polygon_mode)
            .cull_mode(config.cull_mode)
            .front_face(config.front_face)
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .depth_bias_enable(config.depth_bias_enable)
            .line_width(1.0);

        if config.depth_bias_enable {
            rasterization_info = rasterization_info
                .depth_bias_constant_factor(config.depth_bias_constant_factor)
                .depth_bias_slope_factor(config.depth_bias_slope_factor)
        };

        let multisample_info = PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(config.msaa_samples);

        let depth_stencil_info = PipelineDepthStencilStateCreateInfo::default()
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .depth_test_enable(config.depth_test)
            .depth_write_enable(config.depth_write)
            .depth_compare_op(config.depth_compare_op);

        let blend_attachment = if config.blend_enabled {
            let mut state = PipelineColorBlendAttachmentState::default()
                .color_write_mask(config.color_write_mask)
                .blend_enable(true)
                .alpha_blend_op(BlendOp::ADD);

            if let Some(color_blend) = &config.color_blend {
                state = state
                    .color_blend_op(color_blend.blend_op)
                    .src_color_blend_factor(color_blend.src_blend)
                    .dst_color_blend_factor(color_blend.dst_blend)
            };

            if let Some(alpha_blend) = &config.alpha_blend {
                state = state
                    .alpha_blend_op(alpha_blend.blend_op)
                    .src_alpha_blend_factor(alpha_blend.src_blend)
                    .dst_alpha_blend_factor(alpha_blend.dst_blend)
            };

            state
        } else {
            PipelineColorBlendAttachmentState::default()
                .blend_enable(false)
                .color_write_mask(config.color_write_mask)
        };

        let color_blend_info = PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(from_ref(&blend_attachment));

        let dynamic_states = [DynamicState::VIEWPORT, DynamicState::SCISSOR];
        let dynamic_state_info = PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&dynamic_states);

        let mut rendering_info = PipelineRenderingCreateInfo::default()
            .view_mask(config.view_mask)
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
            .layout(self.binding_layout.pipeline_layout_registry.get(PipelineLayoutType::General));

        let pipeline_result = unsafe {
            self.device.create_graphics_pipelines(self.pipeline_cache, &[pipeline_info], None)
        };

        for shader_module in shader_modules {
            unsafe { self.device.destroy_shader_module(shader_module, None) }
        }

        let pipeline = pipeline_result.map(|pipelines| pipelines[0]).unwrap();

        self.debug_utils.label(pipeline, &format!("pipeline_{}", config.label));

        Ok(pipeline)
    }

    fn statistics(&self) -> Self::Statistics {
        ()
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        unsafe { self.device.destroy_pipeline(resource, None) }

        info!("Pipeline destroyed");

        Ok(())
    }
}
