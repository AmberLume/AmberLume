use anyhow::Result;
use ash::Device;
use std::ffi::CString;
use std::sync::Arc;
use ash::vk::{ComputePipelineCreateInfo, Pipeline, PipelineCache, PipelineShaderStageCreateInfo, ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags};
use bytemuck::cast_slice;
use tracing::info;
use crate::render::utils::debug_utils::DebugUtils;
use crate::resources::dynamic::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;
use crate::resources::dynamic::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::dynamic::resource_provider::ResourceId;
use crate::resources::index::resource_index::ResourceIndex;
use crate::resources::persistent::persistent_resources::PersistentResources;

pub struct ComputePipelineBackend {
    device: Device,
    debug_utils: Arc<DebugUtils>,

    resource_index: Arc<ResourceIndex>,

    persistent_resources: Arc<PersistentResources>,

    pipeline_cache: PipelineCache,
}

impl ComputePipelineBackend {
    pub fn new(
        device: Device,
        debug_utils: Arc<DebugUtils>,
        pipeline_cache: PipelineCache,
        resource_index: Arc<ResourceIndex>,
        persistent_resources: Arc<PersistentResources>,
    ) -> Self {
        Self {
            device: device.clone(),
            debug_utils: debug_utils.clone(),

            resource_index,

            persistent_resources,

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

impl ResourceBackend for ComputePipelineBackend {
    type Config = ComputePipelineConfig;
    type Output = Pipeline;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }
    
    fn create(
        &self,
        _id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output> {
        let fn_name = CString::new(config.fn_name.clone()).unwrap();
        let spv = self
            .resource_index
            .get_resource(&config.shader_name)?;

        let shader_module = self.create_shader_module(&config.shader_name, cast_slice(spv))?;
            
        let shader_stage_create_info = PipelineShaderStageCreateInfo::default()
            .name(&fn_name)
            .stage(ShaderStageFlags::COMPUTE)
            .module(shader_module);

        let pipeline_info = ComputePipelineCreateInfo::default()
            .stage(shader_stage_create_info)
            .layout(self.persistent_resources.pipeline_layouts.global);

        let pipeline = unsafe {
            self.device
                .create_compute_pipelines(self.pipeline_cache, &[pipeline_info], None)
                .map(|pipelines| pipelines[0])
                .unwrap()
        };

        self.debug_utils.label(pipeline, &format!("compute_pipeline_{}", config.shader_name));
        
        unsafe { self.device.destroy_shader_module(shader_module, None) };

        Ok(pipeline)
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        unsafe { self.device.destroy_pipeline(resource, None) }

        info!("ComputePipeline destroyed");

        Ok(())
    }
}
