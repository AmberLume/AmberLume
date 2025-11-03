use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::index::resource_index::ResourceIndex;
use crate::resources::io_provider::IOProvider;
use crate::resources::shader::shader_config::ShaderConfig;
use anyhow::Result;
use ash::vk::ShaderModuleCreateInfo;
use ash::{Device, vk};
use bytemuck::cast_slice;
use std::sync::Arc;
use vk::ShaderModule;

pub struct ShaderBackend {
    device: Arc<Device>,

    resource_io: Arc<dyn IOProvider>,
    resource_index: Arc<ResourceIndex>,
}

impl ShaderBackend {
    pub fn new(
        device: Arc<Device>,
        resource_io: Arc<dyn IOProvider>,
        resource_index: Arc<ResourceIndex>,
    ) -> Self {
        Self {
            device,

            resource_io,
            resource_index,
        }
    }
}

pub struct ShaderDependenciesRefs {
    pub shader_name: String,
}

pub struct ShaderDependencies {
    pub spv_src: Vec<u8>,
}

impl ResourceBackend for ShaderBackend {
    type Config = ShaderConfig;
    type DependenciesRefs = ShaderDependenciesRefs;
    type Dependencies = ShaderDependencies;
    type Output = ShaderModule;
    type Storage = ();

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn build_dependencies_refs(&self, config: &Self::Config) -> Self::DependenciesRefs {
        Self::DependenciesRefs {
            shader_name: config.name.clone(),
        }
    }

    fn collect_dependencies(
        &self,
        dependencies_refs: &mut Self::DependenciesRefs,
    ) -> Self::Dependencies {
        let shader_index_entry = self
            .resource_index
            .get_shader_entry(&dependencies_refs.shader_name);

        let shader_data = self
            .resource_io
            .read(&shader_index_entry.file_path)
            .unwrap();

        Self::Dependencies {
            spv_src: shader_data.into(),
        }
    }

    fn create(
        &self,
        config: Self::Config,
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

    fn destroy(&self, resource: Self::Output) -> Result<()> {
        unsafe { self.device.destroy_shader_module(resource, None) }

        Ok(())
    }

    fn get_storage(&self) -> Self::Storage {
        ()
    }
}
