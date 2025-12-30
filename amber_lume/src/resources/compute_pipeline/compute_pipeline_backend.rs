use crate::render::vulkan::device_context::DeviceContext;
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::common::resource_provider::{ResourceId, ResourceProvider};
use crate::resources::pipeline_layout::pipeline_layout_backend::PipelineLayoutBackend;
use crate::resources::shader::shader_backend::ShaderBackend;
use crate::resources::shader::shader_config::ShaderConfig;
use anyhow::Result;
use ash::Device;
use std::ffi::CString;
use std::sync::Arc;
use ash::vk::{ComputePipelineCreateInfo, Pipeline, PipelineCache, PipelineLayout, PipelineShaderStageCreateInfo, ShaderModule, ShaderStageFlags};
use tracing::info;
use crate::render::vulkan::debug_utils::DebugUtils;
use crate::resources::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;

pub struct ComputePipelineBackend {
    device: Device,
    debug_utils: Arc<DebugUtils>,

    shader_provider: Arc<ResourceProvider<ShaderBackend>>,
    pipeline_layout_provider: Arc<ResourceProvider<PipelineLayoutBackend>>,

    pipeline_cache: PipelineCache,
}

impl ComputePipelineBackend {
    pub fn new(
        device_context: &DeviceContext,
        shader_provider: Arc<ResourceProvider<ShaderBackend>>,
        pipeline_layout_provider: Arc<ResourceProvider<PipelineLayoutBackend>>,
        pipeline_cache: PipelineCache,
    ) -> Self {
        Self {
            device: device_context.device.clone(),
            debug_utils: device_context.debug_utils.clone(),

            shader_provider,
            pipeline_layout_provider,

            pipeline_cache,
        }
    }
}

pub struct ComputePipelineDependencies {
    pub shader_stage: ComputePipelineShaderStage,
    pub pipeline_layout: PipelineLayout,
}

pub struct ComputePipelineShaderStage {
    pub fn_name: CString,
    pub shader_module: ShaderModule,
}

impl ResourceBackend for ComputePipelineBackend {
    type Config = ComputePipelineConfig;
    type Dependencies = ComputePipelineDependencies;
    type Output = Pipeline;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn collect_dependencies(&self, config: &Self::Config) -> Self::Dependencies {
        let shader_config = ShaderConfig {
            name: config.shader_name.clone(),
        };

        self.shader_provider.touch(&shader_config);
        let shader_module_id = self.shader_provider.get_id(&shader_config);
        let shader_module = self
            .shader_provider
            .get_ready(&shader_module_id);

        let shader_stage = ComputePipelineShaderStage {
            fn_name: CString::new(config.fn_name.clone()).unwrap(),
            shader_module: *shader_module,
        };

        let pipeline_layout = self
            .pipeline_layout_provider
            .get_now(&config.pipeline_layout_config);

        Self::Dependencies {
            shader_stage,
            pipeline_layout: *pipeline_layout,
        }
    }

    fn create(
        &self,
        _id: &ResourceId,
        config: Self::Config,
        dependencies: Self::Dependencies,
    ) -> Result<Self::Output> {
        let shader_stage_create_info = PipelineShaderStageCreateInfo::default()
            .name(&dependencies.shader_stage.fn_name)
            .stage(ShaderStageFlags::COMPUTE)
            .module(dependencies.shader_stage.shader_module);

        let pipeline_info = ComputePipelineCreateInfo::default()
            .stage(shader_stage_create_info)
            .layout(dependencies.pipeline_layout);

        let pipeline = unsafe {
            self.device
                .create_compute_pipelines(self.pipeline_cache, &[pipeline_info], None)
                .map(|pipelines| pipelines[0])
                .unwrap()
        };

        self.debug_utils.label(pipeline, &format!("{:?}", config));

        Ok(pipeline)
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        unsafe { self.device.destroy_pipeline(resource, None) }

        info!("ComputePipeline destroyed");

        Ok(())
    }
}
