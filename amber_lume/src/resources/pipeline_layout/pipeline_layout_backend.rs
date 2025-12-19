use crate::render::vulkan::device_context::DeviceContext;
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::common::resource_provider::ResourceProvider;
use crate::resources::descriptor_set_layout::descriptor_set_layout_backend::DescriptorSetLayoutBackend;
use crate::resources::pipeline_layout::pipeline_layout_config::PipelineLayoutConfig;
use anyhow::Result;
use ash::Device;
use ash::vk::{DescriptorSetLayout, PipelineLayout, PipelineLayoutCreateInfo, PushConstantRange};
use std::sync::Arc;
use tracing::info;

pub struct PipelineLayoutBackend {
    device: Device,

    descriptor_set_layout_provider: Arc<ResourceProvider<DescriptorSetLayoutBackend>>,
}

impl PipelineLayoutBackend {
    pub fn new(
        device_context: &DeviceContext,
        descriptor_set_layout_provider: Arc<ResourceProvider<DescriptorSetLayoutBackend>>,
    ) -> Self {
        Self {
            device: device_context.device.clone(),

            descriptor_set_layout_provider,
        }
    }
}

pub struct PipelineLayoutDependencies {
    pub descriptor_set_layouts: Vec<DescriptorSetLayout>,
}

impl ResourceBackend for PipelineLayoutBackend {
    type Config = PipelineLayoutConfig;
    type Dependencies = PipelineLayoutDependencies;
    type Output = PipelineLayout;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn collect_dependencies(&self, config: &Self::Config) -> Self::Dependencies {
        let mut descriptor_set_layouts =
            Vec::with_capacity(config.descriptor_set_layout_configs.len());

        for set_layout_config in &config.descriptor_set_layout_configs {
            let set_layout_id = self
                .descriptor_set_layout_provider
                .get_id(&set_layout_config);
            self.descriptor_set_layout_provider.touch(set_layout_config);

            let descriptor_set_layout = self
                .descriptor_set_layout_provider
                .get_ready(&set_layout_id, true)
                .unwrap();

            descriptor_set_layouts.push(*descriptor_set_layout)
        }

        Self::Dependencies {
            descriptor_set_layouts,
        }
    }

    fn create(
        &self,
        config: Self::Config,
        dependencies: Self::Dependencies,
    ) -> Result<Self::Output> {
        let mut push_constant_ranges = Vec::with_capacity(config.push_constant_ranges.len());

        for constant_range in &config.push_constant_ranges {
            let push_constant_range = PushConstantRange::default()
                .stage_flags(constant_range.stage)
                .offset(constant_range.offset)
                .size(constant_range.size);

            push_constant_ranges.push(push_constant_range)
        }

        let layout_info = PipelineLayoutCreateInfo::default()
            .set_layouts(&dependencies.descriptor_set_layouts)
            .push_constant_ranges(&push_constant_ranges);

        let pipeline_layout = unsafe { self.device.create_pipeline_layout(&layout_info, None)? };

        Ok(pipeline_layout)
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        unsafe { self.device.destroy_pipeline_layout(resource, None) }

        info!("Pipeline layout destroyed");

        Ok(())
    }
}
