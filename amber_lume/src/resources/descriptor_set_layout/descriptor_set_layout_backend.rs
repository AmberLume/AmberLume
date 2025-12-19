use crate::render::vulkan::device_context::DeviceContext;
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::descriptor_set_layout::descriptor_set_layout_config::DescriptorSetLayoutConfig;
use anyhow::Result;
use ash::Device;
use ash::vk::{
    DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutBindingFlagsCreateInfo,
    DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo,
};
use tracing::info;

pub struct DescriptorSetLayoutBackend {
    device: Device,
}

impl DescriptorSetLayoutBackend {
    pub fn new(device_context: &DeviceContext) -> Self {
        Self {
            device: device_context.device.clone(),
        }
    }
}

pub struct DescriptorSetLayoutDependencies;

impl ResourceBackend for DescriptorSetLayoutBackend {
    type Config = DescriptorSetLayoutConfig;
    type Dependencies = DescriptorSetLayoutDependencies;
    type Output = DescriptorSetLayout;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn collect_dependencies(&self, _config: &Self::Config) -> Self::Dependencies {
        Self::Dependencies {}
    }

    fn create(
        &self,
        config: Self::Config,
        _dependencies: Self::Dependencies,
    ) -> Result<Self::Output> {
        let mut bindings = Vec::with_capacity(config.bindings.len());
        let mut binding_flags = Vec::with_capacity(config.bindings.len());

        for binding_config in config.bindings {
            let layout_binding = DescriptorSetLayoutBinding::default()
                .binding(binding_config.binding)
                .descriptor_type(binding_config.descriptor_type)
                .descriptor_count(binding_config.descriptor_count)
                .stage_flags(binding_config.stage_flags);

            bindings.push(layout_binding);
            binding_flags.push(binding_config.binding_flags)
        }

        let mut binding_flags_create_info =
            DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&binding_flags);

        let layout_info = DescriptorSetLayoutCreateInfo::default()
            .push_next(&mut binding_flags_create_info)
            .bindings(&bindings)
            .flags(DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);

        let descriptor_set_layout = unsafe {
            self.device
                .create_descriptor_set_layout(&layout_info, None)?
        };

        Ok(descriptor_set_layout)
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        unsafe { self.device.destroy_descriptor_set_layout(resource, None) }

        info!("Descriptor set layout destroyed");

        Ok(())
    }
}
