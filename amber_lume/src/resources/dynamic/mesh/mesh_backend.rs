use crate::render::buffer::buffer_manager::BufferManager;
use crate::resources::index::resource_index::ResourceIndex;
use crate::resources::dynamic::mesh::mesh_config::MeshConfig;
use anyhow::Result;
use rkyv::rancor::Error;
use rkyv::access;
use std::sync::Arc;
use tracing::info;
use builder::data::mesh_data::ArchivedMeshData;
use crate::ids::SliceIndex;
use crate::render::buffer::typed::mesh_buffer::MeshGPU;
use crate::render::buffer::typed::submesh_buffer::SubmeshGPU;
use crate::render::buffer::typed::vertex_buffer::VertexGPU;
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_managers::IndexManagers;
use crate::resources::dynamic::material::material_backend::MaterialBackend;
use crate::resources::dynamic::material::material_config::MaterialConfig;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::dynamic::resource_provider::{ResourceId, ResourceProvider};
use crate::resources::dynamic::skeleton::skeleton_backend::SkeletonBackend;
use crate::resources::dynamic::skeleton::skeleton_config::SkeletonConfig;
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::resources::utils::slice_utils::{as_f32_slice, as_u32_slice};

pub struct MeshBackend {
    buffer_manager: Arc<BufferManager>,
    resource_index: Arc<ResourceIndex>,
    index_managers: Arc<IndexManagers>,

    persistent_resources: Arc<PersistentResources>,

    material_provider: Arc<ResourceProvider<MaterialBackend>>,
    skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,

    resource_loader: Arc<ResourceLoader>,
}

impl MeshBackend {
    pub fn new(
        buffer_manager: Arc<BufferManager>,
        resource_index: Arc<ResourceIndex>,
        index_managers: Arc<IndexManagers>,
        persistent_resources: Arc<PersistentResources>,
        material_provider: Arc<ResourceProvider<MaterialBackend>>,
        skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
        resource_loader: Arc<ResourceLoader>,
    ) -> Self {
        Self {
            buffer_manager,
            resource_index,
            index_managers,

            persistent_resources,

            material_provider,
            skeleton_provider,

            resource_loader,
        }
    }

    fn count_index_vertex_submesh(mesh_data: &ArchivedMeshData) -> (usize, usize, usize) {
        let mut index_count = 0;
        let mut vertex_count = 0;
        let mut submesh_count = 0;

        for submesh_data in mesh_data.submeshes.iter() {
            index_count += submesh_data.indices.len();
            vertex_count += submesh_data.positions.len();
            submesh_count += 1;
        };

        (index_count, vertex_count, submesh_count)
    }
}

pub struct ManagedMesh {
    pub first_index_id: ResourceId,
    pub index_count: u32,
    pub first_vertex_id: ResourceId,
    pub vertex_count: u32,
    pub first_submesh_id: ResourceId,
    pub submesh_count: u32,

    pub skeleton: Option<Arc<ResRef>>,

    pub materials: Vec<Arc<ResRef>>,
}

impl ResourceBackend for MeshBackend {
    type Config = MeshConfig;
    type Output = ManagedMesh;

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output> {
        let mesh_bytes = self.resource_index.get_resource(&config.asset_key)?;
        let archived_mesh_data = access::<ArchivedMeshData, Error>(&mesh_bytes)?;

        let (index_count, vertex_count, submesh_count) = Self::count_index_vertex_submesh(&archived_mesh_data);

        let first_index_id = self.index_managers.index_index_manager.acquire_range(index_count as u32).unwrap();
        let first_vertex_id = self.index_managers.vertex_index_manager.acquire_range(vertex_count as u32).unwrap();
        let first_submesh_id = self.index_managers.submesh_index_manager.acquire_range(submesh_count as u32).unwrap();

        let mut index_id = first_index_id;
        let mut vertex_id = first_vertex_id;
        let mut submesh_id = first_submesh_id;

        let mut materials = Vec::new();

        for submesh_data in archived_mesh_data.submeshes.iter() {
            let indices_count = submesh_data.indices.len() as u32;
            let vertices_count = submesh_data.positions.len() as u32;

            let vertices = (0..submesh_data.positions.iter().count()).map(|index| {
                VertexGPU::from(&submesh_data, index)
            }).collect::<Vec<_>>();

            self.resource_loader.load_buffer_at(
                &self.buffer_manager.index_buffer.slice_at(SliceIndex { value: index_id }),
                as_u32_slice(submesh_data.indices.as_slice()),
            )?;
            self.resource_loader.load_buffer_at(
                &self.buffer_manager.vertex_buffer.slice_at(SliceIndex { value: vertex_id }),
                &vertices,
            )?;

            let material_id = if let Some(resource_key) = submesh_data.material.as_ref() {
                let material_config = MaterialConfig {
                    resource_key: resource_key.value.to_string(),
                };

                let material_res_ref = self.material_provider.get_or_load(material_config);

                materials.push(material_res_ref.clone());

                material_res_ref.id
            } else {
                self.persistent_resources.materials.default.0
            };

            let submesh_gpu = SubmeshGPU::create(
                indices_count,
                index_id,
                vertex_id,
                material_id,
                as_f32_slice(&submesh_data.bounds),
            );
            self.resource_loader.load_buffer_at(
                &self.buffer_manager.submesh_buffer.slice_at(SliceIndex { value: submesh_id }),
                &[submesh_gpu],
            )?;

            index_id += indices_count;
            vertex_id += vertices_count;
            submesh_id += 1;
        }

        let skeleton = archived_mesh_data.skeleton
            .as_ref()
            .map(|skeleton| {
                self.skeleton_provider.get_or_load(SkeletonConfig {
                    resource_key: skeleton.value.to_string(),
                })
            });

        let mesh_gpu = MeshGPU::create(
            first_submesh_id,
            submesh_count as u32,
        );
        self.resource_loader.load_buffer_at(
            &self.buffer_manager.mesh_buffer.slice_at(SliceIndex { value: *id }),
            &[mesh_gpu],
        )?;
        info!("Uploaded mesh: index: {}, data: {:?}", id, mesh_gpu);

        let mesh_allocation = ManagedMesh {
            first_index_id,
            index_count: index_count as u32,
            first_vertex_id,
            vertex_count: vertex_count as u32,
            first_submesh_id,
            submesh_count: submesh_count as u32,

            skeleton,

            materials,
        };

        Ok(mesh_allocation)
    }

    fn set_default(&self, id: &ResourceId) -> Result<()> {
        self.resource_loader.load_buffer_at(
            &self.buffer_manager.mesh_buffer.slice_at(SliceIndex { value: *id }),
            &[self.persistent_resources.meshes.cube.1]
        )?;

        Ok(())
    }

    fn destroy_resource(&self, _resource: Self::Output) -> Result<()> {
        Ok(())
    }
}
