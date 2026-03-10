use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::resources::index::resource_index::ResourceIndex;
use crate::resources::dynamic::model::model_config::ModelConfig;
use anyhow::Result;
use rkyv::rancor::Error;
use rkyv::{access, deserialize};
use std::sync::Arc;
use tracing::info;
use builder::data::model_data::{ArchivedModelData, ModelData};
use crate::ids::SliceIndex;
use crate::render::vulkan::buffer::typed::model_buffer::ModelGpuData;
use crate::render::vulkan::buffer::typed::submesh_buffer::SubmeshGpuData;
use crate::render::vulkan::buffer::typed::vertex_buffer::VertexGpuData;
use crate::render::vulkan::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_managers::IndexManagers;
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
    pub first_submesh_id: ResourceId,
    pub submesh_count: u32,

    pub materials: Vec<Arc<ResRef>>,
}

pub struct ModelBackend {
    buffer_manager: Arc<BufferManager>,
    resource_index: Arc<ResourceIndex>,
    index_managers: Arc<IndexManagers>,

    persistent_resources: Arc<PersistentResources>,

    material_provider: Arc<ResourceProvider<MaterialBackend>>,

    resource_loader: Arc<ResourceLoader>,
}

impl ModelBackend {
    pub fn new(
        buffer_manager: Arc<BufferManager>,
        resource_index: Arc<ResourceIndex>,
        index_manages: Arc<IndexManagers>,
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

    fn count_index_vertex_submesh(model_data: &ModelData) -> (usize, usize, usize) {
        let mut index_count = 0;
        let mut vertex_count = 0;
        let mut submesh_count = 0;

        for mesh_data in &model_data.meshes {
            for submesh_data in &mesh_data.submeshes {
                index_count += submesh_data.indices.len();
                vertex_count += submesh_data.positions.len();
                submesh_count += 1;
            }
        };

        (index_count, vertex_count, submesh_count)
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

        let (index_count, vertex_count, submesh_count) = Self::count_index_vertex_submesh(&model_data);

        let first_index_id = self.index_managers.index_index_manager.acquire_range(index_count as u32).unwrap();
        let first_vertex_id = self.index_managers.vertex_index_manager.acquire_range(vertex_count as u32).unwrap();
        let first_submesh_id = self.index_managers.submesh_index_manager.acquire_range(submesh_count as u32).unwrap();

        let mut index_id = first_index_id;
        let mut vertex_id = first_vertex_id;
        let mut submesh_id = first_submesh_id;

        let mut materials = Vec::new();

        for mesh_data in model_data.meshes {
            for submesh_data in &mesh_data.submeshes {
                let indices_count = submesh_data.indices.len() as u32;
                let vertices_count = submesh_data.positions.len() as u32;

                let vertices = (0..submesh_data.positions.iter().count()).map(|index| {
                    VertexGpuData::from(&submesh_data, index)
                }).collect::<Vec<_>>();

                self.resource_loader.load_buffer_at(
                    &self.buffer_manager.index_buffer.at(SliceIndex { value: index_id }),
                    &submesh_data.indices,
                )?;
                self.resource_loader.load_buffer_at(
                    &self.buffer_manager.vertex_buffer.at(SliceIndex { value: vertex_id }),
                    &vertices,
                )?;

                let material_id = if let Some(material_name) = &submesh_data.material_id {
                    let material_config = MaterialConfig {
                        name: material_name.clone(),
                    };

                    let material_res_ref = self.material_provider.get_or_load(material_config);

                    materials.push(material_res_ref.clone());

                    material_res_ref.id
                } else {
                    self.persistent_resources.materials.default.0
                };

                let submesh_gpu_data = SubmeshGpuData::create(
                    indices_count,
                    index_id,
                    vertex_id,
                    material_id,
                    submesh_data.bounds,
                );
                self.resource_loader.load_buffer_at(
                    &self.buffer_manager.submesh_buffer.at(SliceIndex { value: submesh_id }),
                    &[submesh_gpu_data],
                )?;

                index_id += indices_count;
                vertex_id += vertices_count;
                submesh_id += 1;
            }
        }

        let model_gpu_data = ModelGpuData::create(
            first_submesh_id,
            submesh_count as u32,
        );
        self.resource_loader.load_buffer_at(
            &self.buffer_manager.model_buffer.at(SliceIndex { value: *id }),
            &[model_gpu_data],
        )?;
        info!("Uploaded model: index: {}, data: {:?}", id, model_gpu_data);

        let model_allocation = ModelAllocation {
            first_index_id,
            index_count: index_count as u32,
            first_vertex_id,
            vertex_count: vertex_count as u32,
            first_submesh_id,
            submesh_count: submesh_count as u32,

            materials,
        };

        Ok(model_allocation)
    }

    fn set_default(&self, id: &ResourceId) -> Result<()> {
        self.resource_loader.load_buffer_at(
            &self.buffer_manager.model_buffer.at(SliceIndex { value: *id }),
            &[self.persistent_resources.models.cube.1]
        )?;

        Ok(())
    }
    
    fn destroy_resource(&self, _resource: Self::Output) -> Result<()> {
        Ok(())
    }
}
