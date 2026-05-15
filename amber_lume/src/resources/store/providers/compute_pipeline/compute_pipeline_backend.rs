use anyhow::Result;
use ash::Device;
use std::ffi::CString;
use std::sync::Arc;
use ash::vk::{ComputePipelineCreateInfo, Pipeline, PipelineCache, PipelineShaderStageCreateInfo, ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags, SpecializationInfo, SpecializationMapEntry};
use bytemuck::cast_slice;
use tracing::info;
use crate::render::utils::debug_utils::DebugUtils;
use crate::resources::store::providers::resource_backend::ResourceBackend;
use crate::resources::store::providers::resource_provider::ResourceId;
use crate::resources::alpaca_resource_reader::AlpacaResourceReader;
use crate::resources::binding_layout::binding_layout::BindingLayout;
use crate::resources::binding_layout::pipeline_layout_registry::PipelineLayoutType;
use crate::resources::store::providers::compute_pipeline::compute_pipeline_config::ComputePipelineConfig;

pub struct ComputePipelineBackend {
    device: Device,
    debug_utils: Arc<DebugUtils>,

    alpaca_resource_reader: Arc<AlpacaResourceReader>,

    binding_layout: Arc<BindingLayout>,

    pipeline_cache: PipelineCache,
}

impl ComputePipelineBackend {
    pub fn new(
        device: Device,
        debug_utils: Arc<DebugUtils>,
        pipeline_cache: PipelineCache,
        alpaca_resource_reader: Arc<AlpacaResourceReader>,
        binding_layout: Arc<BindingLayout>,
    ) -> Self {
        Self {
            device,
            debug_utils,

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

impl ResourceBackend for ComputePipelineBackend {
    type Config = ComputePipelineConfig;
    type Output = Pipeline;
    type Statistics = ();

    fn create(
        &self,
        _id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output> {
        let fn_name = CString::new(config.fn_name.clone())?;
        let spv = self
            .alpaca_resource_reader
            .get_resource(config.shader_name.key())?;

        let shader_module = self.create_shader_module(config.shader_name.key(), cast_slice(spv))?;

        let specialization_values: Vec<u32> = config.specialization_entries.iter().map(|(_, v)| *v).collect();
        let specialization_entries: Vec<SpecializationMapEntry> = config.specialization_entries
            .iter()
            .enumerate()
            .map(|(i, (id, _))| {
                SpecializationMapEntry::default()
                    .constant_id(*id)
                    .offset((i * size_of::<u32>()) as u32)
                    .size(size_of::<u32>())
            })
            .collect();
        let spec_data_bytes: &[u8] = cast_slice(&specialization_values);
        let specialization_info = SpecializationInfo::default()
            .map_entries(&specialization_entries)
            .data(spec_data_bytes);

        let mut shader_stage_create_info = PipelineShaderStageCreateInfo::default()
            .name(&fn_name)
            .stage(ShaderStageFlags::COMPUTE)
            .module(shader_module);
        if !config.specialization_entries.is_empty() {
            shader_stage_create_info = shader_stage_create_info
                .specialization_info(&specialization_info);
        }

        let pipeline_info = ComputePipelineCreateInfo::default()
            .stage(shader_stage_create_info)
            .layout(self.binding_layout.pipeline_layout_registry.get(PipelineLayoutType::General));

        let pipeline = unsafe {
            self.device
                .create_compute_pipelines(self.pipeline_cache, &[pipeline_info], None)
                .map(|pipelines| pipelines[0])
                .unwrap()
        };

        self.debug_utils.label(pipeline, &format!("compute_pipeline_{}", config.shader_name.key()));

        unsafe { self.device.destroy_shader_module(shader_module, None) };

        Ok(pipeline)
    }

    fn statistics(&self) -> Self::Statistics {
        ()
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        unsafe { self.device.destroy_pipeline(resource, None) }

        info!("ComputePipeline destroyed");

        Ok(())
    }
}
