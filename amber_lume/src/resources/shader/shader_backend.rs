use crate::render::vulkan::device_context::DeviceContext;
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::index::resource_index::ResourceIndex;
use crate::resources::shader::shader_config::ShaderConfig;
use anyhow::Result;
use ash::vk::ShaderModuleCreateInfo;
use ash::{Device, vk};
use bytemuck::cast_slice;
use std::sync::Arc;
use tracing::info;
use vk::ShaderModule;
use crate::resources::common::resource_provider::ResourceId;

pub struct ShaderBackend {
    device: Device,

    resource_index: Arc<ResourceIndex>,
}

impl ShaderBackend {
    pub fn new(device_context: &DeviceContext, resource_index: Arc<ResourceIndex>) -> Self {
        Self {
            device: device_context.device.clone(),

            resource_index,
        }
    }
}

pub struct ShaderDependencies {
    pub spv_src: Vec<u8>,
}

impl ResourceBackend for ShaderBackend {
    type Config = ShaderConfig;
    type Dependencies = ShaderDependencies;
    type Output = ShaderModule;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn collect_dependencies(&self, config: &Self::Config) -> Self::Dependencies {
        let shader_slice = self.resource_index.get_resource(&config.name).unwrap();

        Self::Dependencies {
            spv_src: Vec::from(shader_slice),
        }
    }

    fn create(
        &self,
        _id: &ResourceId,
        _config: Self::Config,
        dependencies: Self::Dependencies,
    ) -> Result<Self::Output> {
        let shader_module_create_info =
            ShaderModuleCreateInfo::default().code(cast_slice(&dependencies.spv_src));

        let shader_module = unsafe {
            self.device
                .create_shader_module(&shader_module_create_info, None)?
        };

        Ok(shader_module)
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        unsafe { self.device.destroy_shader_module(resource, None) }

        info!("Shader destroyed");

        Ok(())
    }
}
