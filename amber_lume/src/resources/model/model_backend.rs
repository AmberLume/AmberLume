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
use std::sync::atomic::{AtomicU32, Ordering};
use ash::vk::DeviceSize;
use bytemuck::bytes_of;
use tracing::info;
use crate::render::vulkan::buffer::typed::model_buffer::ModelGpuData;
use crate::render::vulkan::buffer::typed::primitive_buffer::PrimitiveGpuData;
use crate::render::vulkan::buffer::typed::vertex_buffer::VertexGpuData;
use crate::resources::common::resource_provider::ResourceId;

pub struct ModelBackend {
    small_transfer_context: Arc<Mutex<TransferContext>>,

    primitive_count: AtomicU32,

    buffer_manager: Arc<BufferManager>,

    resource_index: Arc<ResourceIndex>,
}

impl ModelBackend {
    pub fn new(
        resource_context: &ResourceContext,
        resource_index: Arc<ResourceIndex>,
    ) -> Self {
        Self {
            small_transfer_context: resource_context.small_transfer_context.clone(),

            primitive_count: AtomicU32::new(0),

            buffer_manager: resource_context.buffer_manager.clone(),

            resource_index,
        }
    }

    fn count_index_vertex_primitive(model_data: &ModelData) -> (usize, usize, usize) {
        let mut index_count = 0;
        let mut vertex_count = 0;
        let mut primitive_count = 0;

        for mesh_data in &model_data.meshes {
            for primitive_data in &mesh_data.primitives {
                index_count += primitive_data.indices.len();
                vertex_count += primitive_data.vertices.len();
                primitive_count += 1;
            }
        };

        (index_count, vertex_count, primitive_count)
    }

    fn upload_indices_vertices(
        &self,
        transfer_context: &mut TransferContext,
        primitive_data: &PrimitiveData,
        index_offset: u32,
        vertex_offset: u32,
        index_offset_bytes: DeviceSize,
        vertex_offset_bytes: DeviceSize,
    ) -> Result<PrimitiveGpuData> {
        let vertices = (0..primitive_data.vertices.iter().count()).map(|index| {
            VertexGpuData::from(&primitive_data, index)
        }).collect::<Vec<_>>();

        transfer_context.copy_staged_at(
            &self.buffer_manager.index_buffer,
            index_offset_bytes + (index_offset * size_of::<u32>() as u32) as DeviceSize,
            &primitive_data.indices,
        )?;

        transfer_context.copy_staged_at(
            &self.buffer_manager.vertex_buffer,
            vertex_offset_bytes + (vertex_offset * size_of::<VertexGpuData>() as u32) as DeviceSize,
            &vertices,
        )?;

        let primitive_gpu_data = PrimitiveGpuData::create(
            primitive_data.indices.len() as u32,
            index_offset,
            vertex_offset,
        );

        info!("Uploaded indices and vertices: {:?}", primitive_gpu_data);

        Ok(primitive_gpu_data)
    }

    fn upload_primitive(&self, index: u32, primitive_gpu_data: &PrimitiveGpuData) -> Result<()> {
        let offset = size_of::<PrimitiveGpuData>() * index as usize;

        self.buffer_manager.primitive_buffer.stage(offset as DeviceSize, &bytes_of(primitive_gpu_data))?;

        info!("Uploaded primitive: index: {}, data: {:?}", index, primitive_gpu_data);

        Ok(())
    }

    fn upload_model(&self, index: u32, model_gpu_data: &ModelGpuData) -> Result<()> {
        let offset = size_of::<ModelGpuData>() * index as usize;

        self.buffer_manager.model_buffer.stage(offset as DeviceSize, &bytes_of(model_gpu_data))?;
        info!("Uploaded model: index: {}, data: {:?}", index, model_gpu_data);

        self.buffer_manager.model_availability_buffer.stage(index as DeviceSize, &[1u8])?;
        info!("Model resource {} is now available", index);

        Ok(())
    }
}

pub struct ModelDependencies {
    pub model_data: ModelData,
}

impl ResourceBackend for ModelBackend {
    type Config = ModelConfig;
    type Dependencies = ModelDependencies;
    type Output = ();

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
        id: &ResourceId,
        _config: Self::Config,
        dependencies: Self::Dependencies,
    ) -> Result<Self::Output> {
        let mut transfer_context = self.small_transfer_context.lock().unwrap();

        transfer_context.begin()?;

        let (index_count, vertex_count, primitive_count) = Self::count_index_vertex_primitive(&dependencies.model_data);
        let index_offset_bytes = self.buffer_manager.index_buffer.allocate_space_for(index_count)?;
        let vertex_offset_bytes =self.buffer_manager.vertex_buffer.allocate_space_for(vertex_count)?;

        let primitive_offset = self.primitive_count.fetch_add(primitive_count as u32, Ordering::Relaxed);
        let mut current_primitive_index: u32 = primitive_offset;

        let mut index_offset: u32 = 0;
        let mut vertex_offset: u32 = 0;

        for mesh_data in dependencies.model_data.meshes {
            for primitive_data in &mesh_data.primitives {
                let primitive_gpu_data = self.upload_indices_vertices(
                    &mut transfer_context,
                    &primitive_data,
                    index_offset,
                    vertex_offset,
                    index_offset_bytes,
                    vertex_offset_bytes,
                )?;
                index_offset += primitive_data.indices.len() as u32;
                vertex_offset += primitive_data.vertices.len() as u32;

                self.upload_primitive(current_primitive_index, &primitive_gpu_data)?;
                current_primitive_index += 1;
            }
        }

        let model_gpu_data = ModelGpuData::create(
            primitive_offset,
            primitive_count as u32,
        );
        self.upload_model(*id, &model_gpu_data)?;

        transfer_context.submit()?;

        Ok(())
    }

    fn destroy_resource(&self, _resource: Self::Output) -> Result<()> {
        Ok(())
    }

    fn destroy(&mut self) -> Result<()> {
        Ok(())
    }
}
