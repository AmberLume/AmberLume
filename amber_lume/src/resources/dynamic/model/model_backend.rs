use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::resources::index::resource_index::ResourceIndex;
use crate::resources::dynamic::model::model_config::ModelConfig;
use anyhow::Result;
use rkyv::rancor::Error;
use rkyv::{access, deserialize};
use std::sync::Arc;
use tracing::info;
use builder::data::model_data::{ArchivedModelData, ModelData};
use crate::render::vulkan::buffer::typed::model_buffer::ModelGpuData;
use crate::render::vulkan::buffer::typed::primitive_buffer::PrimitiveGpuData;
use crate::render::vulkan::buffer::typed::vertex_buffer::VertexGpuData;
use crate::render::vulkan::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_managers::DescriptorIndexManagers;
use crate::resources::dynamic::material::material_backend::MaterialBackend;
use crate::resources::dynamic::material::material_config::MaterialConfig;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::dynamic::resource_provider::{ResourceId, ResourceProvider};
use crate::resources::persistent::persistent_resources::PersistentResources;

pub struct ModelAllocation {
    pub first_index_id: ResourceId,
    pub index_count: u32,
    pub first_vertex_id: ResourceId,
    pub vertex_count: u32,
    pub first_primitive_id: ResourceId,
    pub primitive_count: u32,

    pub materials: Vec<Arc<ResRef>>,
}

pub struct ModelBackend {
    buffer_manager: Arc<BufferManager>,
    resource_index: Arc<ResourceIndex>,
    index_managers: Arc<DescriptorIndexManagers>,

    persistent_resources: Arc<PersistentResources>,

    material_provider: Arc<ResourceProvider<MaterialBackend>>,

    resource_loader: Arc<ResourceLoader>,
}

impl ModelBackend {
    pub fn new(
        buffer_manager: Arc<BufferManager>,
        resource_index: Arc<ResourceIndex>,
        index_manages: Arc<DescriptorIndexManagers>,
        persistent_resources: Arc<PersistentResources>,
        material_provider: Arc<ResourceProvider<MaterialBackend>>,
        resource_loader: Arc<ResourceLoader>,
    ) -> Self {
        Self {
            buffer_manager,
            resource_index,
            index_managers: index_manages,

            persistent_resources,

            material_provider,

            resource_loader,
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
}

impl ResourceBackend for ModelBackend {
    type Config = ModelConfig;
    type Output = ModelAllocation;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output> {
        let mesh_bytes = self.resource_index.get_resource(&config.name)?;

        let archived = access::<ArchivedModelData, Error>(&mesh_bytes)?;

        let model_data = deserialize::<ModelData, Error>(archived)?;

        let (index_count, vertex_count, primitive_count) = Self::count_index_vertex_primitive(&model_data);

        let first_index_id = self.index_managers.index_index_manager.acquire_range(index_count as u32).unwrap();
        let first_vertex_id = self.index_managers.vertex_index_manager.acquire_range(vertex_count as u32).unwrap();
        let first_primitive_id = self.index_managers.primitive_index_manager.acquire_range(primitive_count as u32).unwrap();

        let mut index_id = first_index_id;
        let mut vertex_id = first_vertex_id;
        let mut primitive_id = first_primitive_id;

        let mut materials = Vec::new();

        for mesh_data in model_data.meshes {
            for primitive_data in &mesh_data.primitives {
                let indices_count = primitive_data.indices.len() as u32;
                let vertices_count = primitive_data.positions.len() as u32;

                let vertices = (0..primitive_data.positions.iter().count()).map(|index| {
                    VertexGpuData::from(&primitive_data, index)
                }).collect::<Vec<_>>();

                self.resource_loader.load_buffer_at(
                    &self.buffer_manager.index_buffer,
                    index_id,
                    &primitive_data.indices,
                )?;
                self.resource_loader.load_buffer_at(
                    &self.buffer_manager.vertex_buffer,
                    vertex_id,
                    &vertices,
                )?;

                let material_id = if let Some(material_name) = &primitive_data.material_id {
                    let material_config = MaterialConfig {
                        name: material_name.clone(),
                    };

                    let material_res_ref = self.material_provider.get_or_load(material_config);

                    materials.push(material_res_ref.clone());

                    material_res_ref.id
                } else {
                    self.persistent_resources.materials.default.0
                };

                let primitive_gpu_data = PrimitiveGpuData::create(
                    indices_count,
                    index_id,
                    vertex_id,
                    material_id,
                );
                self.resource_loader.load_buffer_at(
                    &self.buffer_manager.primitive_buffer,
                    primitive_id,
                    &[primitive_gpu_data],
                )?;

                index_id += indices_count;
                vertex_id += vertices_count;
                primitive_id += 1;
            }
        }

        let model_gpu_data = ModelGpuData::create(
            first_primitive_id,
            primitive_count as u32,
        );
        self.resource_loader.load_buffer_at(
            &self.buffer_manager.model_buffer,
            *id,
            &[model_gpu_data],
        )?;
        info!("Uploaded model: index: {}, data: {:?}", id, model_gpu_data);

        let model_allocation = ModelAllocation {
            first_index_id,
            index_count: index_count as u32,
            first_vertex_id,
            vertex_count: vertex_count as u32,
            first_primitive_id,
            primitive_count: primitive_count as u32,

            materials,
        };

        Ok(model_allocation)
    }

    fn set_default(&self, id: &ResourceId) -> Result<()> {
        self.resource_loader.load_buffer_at(
            &self.buffer_manager.model_buffer,
            *id,
            &[self.persistent_resources.models.cube.1]
        )?;

        Ok(())
    }
    
    fn destroy_resource(&self, _resource: Self::Output) -> Result<()> {
        Ok(())
    }
}
