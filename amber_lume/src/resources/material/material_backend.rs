use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::buffer::resource_context::ResourceContext;
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::index::resource_index::ResourceIndex;
use anyhow::Result;
use rkyv::rancor::Error;
use rkyv::{access, deserialize};
use std::sync::Arc;
use ash::vk::DeviceSize;
use bytemuck::bytes_of;
use tracing::info;
use alpaca::data::common::material_data::{ArchivedMaterialData, MaterialData};
use crate::render::vulkan::buffer::typed::material_buffer::MaterialGpuData;
use crate::resources::common::resource_provider::ResourceId;
use crate::resources::material::material_config::MaterialConfig;

pub struct MaterialBackend {
    buffer_manager: Arc<BufferManager>,

    resource_index: Arc<ResourceIndex>,
}

impl MaterialBackend {
    pub fn new(
        resource_context: &ResourceContext,
        resource_index: Arc<ResourceIndex>,
    ) -> Self {
        Self {
            buffer_manager: resource_context.buffer_manager.clone(),

            resource_index,
        }
    }

    fn upload_material(&self, index: u32, material_gpu_data: &MaterialGpuData) -> Result<()> {
        let offset = size_of::<MaterialGpuData>() * index as usize;

        self.buffer_manager.material_buffer.stage(offset as DeviceSize, &bytes_of(material_gpu_data))?;
        info!("Uploaded material: index: {}, data: {:?}", index, material_gpu_data);

        self.buffer_manager.material_availability_buffer.stage(index as DeviceSize, &[1u8])?;
        info!("Material resource {} is now available", index);

        Ok(())
    }
}

pub struct MaterialDependencies {
    pub material_data: MaterialData,
}

impl ResourceBackend for MaterialBackend {
    type Config = MaterialConfig;
    type Dependencies = MaterialDependencies;
    type Output = ();

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn collect_dependencies(&self, config: &Self::Config) -> Self::Dependencies {
        let material_bytes = self.resource_index.get_resource(&config.name).unwrap();

        let archived = access::<ArchivedMaterialData, Error>(&material_bytes).unwrap();

        let material_data = deserialize::<MaterialData, Error>(archived).unwrap();

        Self::Dependencies { material_data }
    }

    fn create(
        &self,
        id: &ResourceId,
        _config: Self::Config,
        dependencies: Self::Dependencies,
    ) -> Result<Self::Output> {
        let material_data = MaterialGpuData::create(
            dependencies.material_data.base_color,
            0,
        );

        self.upload_material(*id, &material_data)?;

        Ok(())
    }

    fn destroy_resource(&self, _resource: Self::Output) -> Result<()> {
        Ok(())
    }

    fn destroy(&mut self) -> Result<()> {
        Ok(())
    }
}
