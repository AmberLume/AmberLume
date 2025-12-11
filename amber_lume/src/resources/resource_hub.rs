use crate::render::vulkan::device_context::DeviceContext;
use crate::resources::common::resource_provider::ResourceProvider;
use crate::resources::index::resource_index::ResourceIndex;
use crate::resources::providers::io_provider::IOProvider;
use crate::resources::shader::shader_backend::ShaderBackend;
use anyhow::Result;
use std::sync::Arc;

pub struct ResourceHub {
    shader_provider: Arc<ResourceProvider<ShaderBackend>>,
}

impl ResourceHub {
    pub fn new(device_context: &DeviceContext, io_provider: Arc<dyn IOProvider>) -> Result<Self> {
        let resource_index = {
            let resource_index = ResourceIndex::new(io_provider.clone())?;

            Arc::new(resource_index)
        };

        let shader_provider = {
            let shader_backend = ShaderBackend::new(&device_context, resource_index);

            ResourceProvider::from(shader_backend)
        };

        Ok(Self { shader_provider })
    }

    pub fn get_shader_provider(&self) -> Arc<ResourceProvider<ShaderBackend>> {
        self.shader_provider.clone()
    }

    pub fn destroy(&self) -> Result<()> {
        self.shader_provider.destroy()
    }
}
