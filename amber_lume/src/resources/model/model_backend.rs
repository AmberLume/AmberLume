use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::buffer::resource_context::ResourceContext;
use crate::render::vulkan::buffer::transfer_context::TransferContext;
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::index::resource_index::ResourceIndex;
use crate::resources::model::model_config::ModelConfig;
use alpaca::data::common::model_data::{ArchivedModelData, ModelData};
use alpaca::data::common::primitive_data::PrimitiveData;
use anyhow::Result;
use rkyv::rancor::Error;
use rkyv::{access, deserialize};
use std::sync::{Arc, Mutex};
use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::buffer::typed::vertex_buffer::VertexGpuData;

pub struct ModelBackend {
    small_transfer_context: Arc<Mutex<TransferContext>>,

    index_buffer: Arc<Buffer>,
    vertex_buffer: Arc<Buffer>,

    resource_index: Arc<ResourceIndex>,
}

impl ModelBackend {
    pub fn new(
        resource_context: &ResourceContext,
        buffer_manager: &BufferManager,
        resource_index: Arc<ResourceIndex>,
    ) -> Self {
        Self {
            small_transfer_context: resource_context.small_transfer_context.clone(),

            index_buffer: buffer_manager.index_buffer.clone(),
            vertex_buffer: buffer_manager.vertex_buffer.clone(),

            resource_index,
        }
    }

    fn upload_primitive(
        &self,
        transfer_context: &mut TransferContext,
        primitive_data: &PrimitiveData,
    ) -> Result<PrimitiveAllocation> {
        let mut vertices = Vec::with_capacity(primitive_data.vertices.len());

        for index in 0..primitive_data.vertices.iter().count() {
            vertices.push(VertexGpuData::from(&primitive_data, index));
        }

        let indices_offset = {
            let indices_bytes_offset = self.index_buffer.allocate_space_for(primitive_data.indices.len())?;

            transfer_context.copy_staged_at(
                &self.index_buffer,
                indices_bytes_offset,
                &primitive_data.indices,
            )?;

            indices_bytes_offset / size_of::<u32>() as u64
        };

        let vertices_offset = {
            let vertices_bytes_offset = self.vertex_buffer.allocate_space_for(vertices.len())?;

            transfer_context.copy_staged_at(
                &self.vertex_buffer,
                vertices_bytes_offset,
                &vertices,
            )?;

            vertices_bytes_offset / size_of::<VertexGpuData>() as u64
        };

        let primitive_allocation = PrimitiveAllocation {
            index_offset: indices_offset as u32,
            index_count: primitive_data.indices.len() as u32,
            vertex_offset: vertices_offset as i32,
        };

        Ok(primitive_allocation)
    }
}

pub struct ModelDependencies {
    pub model_data: ModelData,
}

#[derive(Debug, Clone)]
pub struct PrimitiveAllocation {
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: i32,
}

impl ResourceBackend for ModelBackend {
    type Config = ModelConfig;
    type Dependencies = ModelDependencies;
    type Output = Vec<PrimitiveAllocation>;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn collect_dependencies(&self, config: &Self::Config) -> Self::Dependencies {
        let mesh_bytes = self.resource_index.get_resource(&config.name).unwrap();

        let archived = access::<ArchivedModelData, Error>(&mesh_bytes).unwrap();

        let model_data = deserialize::<ModelData, Error>(archived).unwrap();

        Self::Dependencies { model_data }
    }

    fn create(
        &self,
        _config: Self::Config,
        dependencies: Self::Dependencies,
    ) -> Result<Self::Output> {
        let mut transfer_context = self.small_transfer_context.lock().unwrap();

        transfer_context.begin()?;

        let mut primitive_allocations = Vec::new();

        for mesh_data in dependencies.model_data.meshes {
            for primitive_data in mesh_data.primitives {
                let primitive_allocation =
                    self.upload_primitive(&mut transfer_context, &primitive_data)?;

                primitive_allocations.push(primitive_allocation);
            }
        }

        transfer_context.submit()?;

        println!("Primitive allocations: {:?}", primitive_allocations);

        Ok(primitive_allocations)
    }

    fn destroy_resource(&self, _resource: Self::Output) -> Result<()> {
        Ok(())
    }

    fn destroy(&mut self) -> Result<()> {
        Ok(())
    }
}
