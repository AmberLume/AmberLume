use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::render::vulkan::buffer::transfer_context::TransferContext;
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::index::resource_index::ResourceIndex;
use crate::resources::model::model_config::ModelConfig;
use anyhow::{bail, Result};
use rkyv::rancor::Error;
use rkyv::{access, deserialize};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use ash::vk::DeviceSize;
use bytemuck::bytes_of;
use tracing::info;
use builder::data::model_data::{ArchivedModelData, ModelData};
use builder::data::primitive_data::PrimitiveData;
use crate::render::vulkan::buffer::typed::model_buffer::ModelGpuData;
use crate::render::vulkan::buffer::typed::primitive_buffer::PrimitiveGpuData;
use crate::render::vulkan::buffer::typed::vertex_buffer::VertexGpuData;
use crate::resources::common::resource_provider::{ResourceId, ResourceProvider};
use crate::resources::material::material_backend::MaterialBackend;
use crate::resources::material::material_config::MaterialConfig;
use crate::resources::res_ref::ResRef;

pub struct ModelBackend {
    large_transfer_context: Arc<Mutex<Option<TransferContext>>>,

    primitive_count: AtomicU32,

    material_provider: Arc<ResourceProvider<MaterialBackend>>,

    buffer_manager: Arc<BufferManager>,

    resource_index: Arc<ResourceIndex>,
}

impl ModelBackend {
    pub fn new(
        resource_context: &ResourceContext,
        resource_index: Arc<ResourceIndex>,
        material_provider: Arc<ResourceProvider<MaterialBackend>>,
    ) -> Self {
        Self {
            large_transfer_context: resource_context.large_transfer_context.clone(),

            primitive_count: AtomicU32::new(0),

            material_provider,

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
                vertex_count += primitive_data.positions.len();
                primitive_count += 1;
            }
        };

        (index_count, vertex_count, primitive_count)
    }

    fn get_material_id(&self, primitive_data: &PrimitiveData) -> Result<ResourceId> {
        if let Some(material_id) = primitive_data.material_id.clone() {
            let mut material_resref = ResRef::from(MaterialConfig {
                name: material_id,
            });

            self.material_provider.ensure(&mut material_resref);

            Ok(material_resref.get_id().unwrap())
        } else {
            Ok(!0)
        }
    }

    fn upload_indices(
        &self,
        transfer_context: &mut TransferContext,
        primitive_data: &PrimitiveData,
        index_offset: usize,
    ) -> Result<()> {
        let source_offset = transfer_context.stage(&primitive_data.indices)?;
        transfer_context.flush_to_buffer(
            &self.buffer_manager.index_buffer.handle,
            source_offset,
            index_offset as DeviceSize * self.buffer_manager.index_buffer.item_size,
        )?;

        Ok(())
    }

    fn upload_vertices(
        &self,
        transfer_context: &mut TransferContext,
        primitive_data: &PrimitiveData,
        vertex_offset: usize,
    ) -> Result<()> {
        let vertices = (0..primitive_data.positions.iter().count()).map(|index| {
            VertexGpuData::from(&primitive_data, index)
        }).collect::<Vec<_>>();

        let source_offset = transfer_context.stage(&vertices)?;
        transfer_context.flush_to_buffer(
            &self.buffer_manager.vertex_buffer.handle,
            source_offset,
            vertex_offset as DeviceSize * self.buffer_manager.vertex_buffer.item_size,
        )?;

        Ok(())
    }

    fn upload_indices_vertices(
        &self,
        transfer_context: &mut TransferContext,
        primitive_data: &PrimitiveData,
        index_offset: usize,
        vertex_offset: usize,
    ) -> Result<PrimitiveGpuData> {
        self.upload_indices(transfer_context, primitive_data, index_offset)?;
        self.upload_vertices(transfer_context, primitive_data, vertex_offset)?;
        let material_id = self.get_material_id(&primitive_data)?;

        let primitive_gpu_data = PrimitiveGpuData::create(
            primitive_data.indices.len() as u32,
            index_offset as u32,
            vertex_offset as u32,
            material_id,
        );

        info!("Uploaded indices and vertices: {:?}", primitive_gpu_data);

        Ok(primitive_gpu_data)
    }

    fn upload_primitive(&self, index: u32, primitive_gpu_data: &PrimitiveGpuData) -> Result<()> {
        self.buffer_manager.primitive_buffer.stage(index as usize, &bytes_of(primitive_gpu_data))?;

        info!("Uploaded primitive: index: {}, data: {:?}", index, primitive_gpu_data);

        Ok(())
    }

    fn upload_model(&self, index: u32, model_gpu_data: &ModelGpuData) -> Result<()> {
        self.buffer_manager.model_buffer.stage(index as usize, &bytes_of(model_gpu_data))?;
        info!("Uploaded model: index: {}, data: {:?}", index, model_gpu_data);

        self.buffer_manager.model_availability_buffer.stage(index as usize, &[1u32])?;
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
        let mut transfer_context_guard = self.large_transfer_context.lock().unwrap();
        let Some(transfer_context) = transfer_context_guard.as_mut() else {
            bail!("Transfer context is None")
        };

        transfer_context.begin()?;

        let (index_count, vertex_count, primitive_count) = Self::count_index_vertex_primitive(&dependencies.model_data);
        let mut index_offset = self.buffer_manager.index_buffer.allocate_space_for(index_count)?;
        let mut vertex_offset = self.buffer_manager.vertex_buffer.allocate_space_for(vertex_count)?;

        let primitive_offset = self.primitive_count.fetch_add(primitive_count as u32, Ordering::Relaxed);
        let mut current_primitive_index: u32 = primitive_offset;

        for mesh_data in dependencies.model_data.meshes {
            for primitive_data in &mesh_data.primitives {
                let primitive_gpu_data = self.upload_indices_vertices(
                    transfer_context,
                    &primitive_data,
                    index_offset,
                    vertex_offset,
                )?;
                index_offset += primitive_data.indices.len();
                vertex_offset += primitive_data.positions.len();

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
