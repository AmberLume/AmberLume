use std::slice::Iter;
use crate::render::buffer::buffer_manager::BufferManager;
use crate::resources::dynamic::mesh::mesh_config::{MeshConfig, SubmeshConfig};
use anyhow::Result;
use rkyv::rancor::Error;
use rkyv::access;
use std::sync::Arc;
use tracing::info;
use builder::data::mesh_data::ArchivedMeshData;
use builder::data::submesh_data::ArchivedSubmeshData;
use crate::ids::SliceIndex;
use crate::render::buffer::typed::mesh_buffer::MeshGPU;
use crate::render::buffer::typed::submesh_buffer::SubmeshGPU;
use crate::render::buffer::typed::vertex_buffer::VertexGPU;
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::alpaca_resource_reader::alpaca_resource_reader::AlpacaResourceReader;
use crate::resources::descriptor_index_managers::IndexManagers;
use crate::resources::dynamic::material::material_backend::MaterialBackend;
use crate::resources::dynamic::material::material_config::MaterialConfig;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::dynamic::resource_provider::{ResourceId, ResourceProvider};
use crate::resources::dynamic::skeleton::skeleton_backend::SkeletonBackend;
use crate::resources::dynamic::skeleton::skeleton_config::SkeletonConfig;
use crate::resources::persistent::persistent_materials::PersistentMaterials;
use crate::resources::utils::slice_utils::{as_f32_slice};

pub struct MeshBackend {
    buffer_manager: Arc<BufferManager>,
    alpaca_resource_reader: Arc<AlpacaResourceReader>,
    index_managers: Arc<IndexManagers>,

    material_provider: Arc<ResourceProvider<MaterialBackend>>,
    skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,

    default_material: Arc<ResRef>,

    resource_loader: Arc<ResourceLoader>,
}

impl MeshBackend {
    pub fn new(
        buffer_manager: Arc<BufferManager>,
        alpaca_resource_reader: Arc<AlpacaResourceReader>,
        index_managers: Arc<IndexManagers>,
        material_provider: Arc<ResourceProvider<MaterialBackend>>,
        skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
        persistent_materials: &PersistentMaterials,
        resource_loader: Arc<ResourceLoader>,
    ) -> Self {
        Self {
            buffer_manager,
            alpaca_resource_reader,
            index_managers,

            material_provider,
            skeleton_provider,

            default_material: persistent_materials.default.clone(),

            resource_loader,
        }
    }

    fn count_archived_index_vertex_submesh(data: &ArchivedMeshData) -> (u32, u32, u32) {
        let mut index_count: u32 = 0;
        let mut vertex_count: u32 = 0;
        let mut submesh_count: u32 = 0;

        for submesh_data in data.submeshes.iter() {
            index_count += submesh_data.indices.len() as u32;
            vertex_count += submesh_data.positions.len() as u32;
            submesh_count += 1;
        };

        (index_count, vertex_count, submesh_count)
    }

    fn count_config_index_vertex_submesh(configs: &[SubmeshConfig]) -> (u32, u32, u32) {
        let mut index_count: u32 = 0;
        let mut vertex_count: u32 = 0;
        let mut submesh_count: u32 = 0;

        for submesh_config in configs {
            index_count += submesh_config.indices.len() as u32;
            vertex_count += submesh_config.vertices.len() as u32;
            submesh_count += 1;
        };

        (index_count, vertex_count, submesh_count)
    }

    fn extract_archived_submeshes(&self, submeshes: Iter<'_, ArchivedSubmeshData>) -> Vec<(Vec<u32>, Vec<VertexGPU>, Arc<ResRef>, [f32; 6])> {
        submeshes.map(|submesh_data| {
            let indices = submesh_data.indices.iter()
                .map(|v| v.to_native())
                .collect::<Vec<_>>();
            let vertices = submesh_data.positions.iter().enumerate().map(|(index, _)| {
                VertexGPU::from(&submesh_data, index)
            }).collect::<Vec<_>>();

            let material = if let Some(resource_key) = submesh_data.material.as_ref() {
                self.material_provider.get_or_load(MaterialConfig::Alpaca {
                    resource_key: resource_key.value.to_string(),
                })
            } else {
                self.default_material.clone()
            };

            (indices, vertices, material, as_f32_slice(&submesh_data.bounds))
        }).collect::<Vec<_>>()
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
        match config {
            Self::Config::Alpaca { resource_key } => {
                let mesh_bytes = self.alpaca_resource_reader.get_resource(&resource_key)?;
                let archived_mesh_data = access::<ArchivedMeshData, Error>(&mesh_bytes)?;

                let mut materials: Vec<Arc<ResRef>> = Vec::new();

                let (index_count, vertex_count, submesh_count) = Self::count_archived_index_vertex_submesh(&archived_mesh_data);

                let first_index_id = self.index_managers.index_index_manager.acquire_range(index_count).unwrap();
                let first_vertex_id = self.index_managers.vertex_index_manager.acquire_range(vertex_count).unwrap();
                let first_submesh_id = self.index_managers.submesh_index_manager.acquire_range(submesh_count).unwrap();

                let mut index_id = first_index_id;
                let mut vertex_id = first_vertex_id;
                let mut submesh_id = first_submesh_id;

                let submeshes = self.extract_archived_submeshes(archived_mesh_data.submeshes.iter());

                for (indices, vertices, material, aabb) in submeshes {
                    self.resource_loader.load_buffer_at(
                        &self.buffer_manager.index_buffer.slice_at(SliceIndex::from(index_id)),
                        &indices,
                    )?;
                    self.resource_loader.load_buffer_at(
                        &self.buffer_manager.vertex_buffer.slice_at(SliceIndex::from(vertex_id)),
                        &vertices,
                    )?;

                    materials.push(material.clone());

                    let submesh = SubmeshGPU::create(
                        indices.len() as u32,
                        index_id,
                        vertex_id,
                        material.id,
                        aabb,
                    );

                    self.resource_loader.load_buffer_at(
                        &self.buffer_manager.submesh_buffer.slice_at(SliceIndex::from(submesh_id)),
                        &[submesh],
                    )?;

                    index_id += indices.len() as u32;
                    vertex_id += vertices.len() as u32;
                    submesh_id += 1;
                }

                let skeleton = archived_mesh_data.skeleton
                    .as_ref()
                    .map(|skeleton| {
                        self.skeleton_provider.get_or_load(SkeletonConfig::Alpaca {
                            resource_key: skeleton.value.to_string(),
                        })
                    });

                let mesh_gpu_data = MeshGPU::create(
                    first_submesh_id,
                    submesh_count,
                );

                self.resource_loader.load_buffer_at(
                    &self.buffer_manager.mesh_buffer.slice_at(SliceIndex::from(*id)),
                    &[mesh_gpu_data],
                )?;
                info!("Uploaded mesh: index: {}, data: {:?}", id, mesh_gpu_data);

                Ok(ManagedMesh {
                    first_index_id,
                    index_count,
                    first_vertex_id,
                    vertex_count,
                    first_submesh_id,
                    submesh_count,

                    skeleton,

                    materials,
                })
            }
            Self::Config::InBuilt { submeshes, skeleton } => {
                let (index_count, vertex_count, submesh_count) = Self::count_config_index_vertex_submesh(&submeshes);

                let mut materials: Vec<Arc<ResRef>> = Vec::new();

                let first_index_id = self.index_managers.index_index_manager.acquire_range(index_count).unwrap();
                let first_vertex_id = self.index_managers.vertex_index_manager.acquire_range(vertex_count).unwrap();
                let first_submesh_id = self.index_managers.submesh_index_manager.acquire_range(submesh_count).unwrap();

                let mut index_id = first_index_id;
                let mut vertex_id = first_vertex_id;
                let mut submesh_id = first_submesh_id;

                for submesh_config in submeshes {
                    let indices_count = submesh_config.indices.len() as u32;
                    let vertices_count = submesh_config.vertices.len() as u32;

                    self.resource_loader.load_buffer_at(
                        &self.buffer_manager.index_buffer.slice_at(SliceIndex::from(index_id)),
                        &submesh_config.indices,
                    )?;
                    self.resource_loader.load_buffer_at(
                        &self.buffer_manager.vertex_buffer.slice_at(SliceIndex::from(vertex_id)),
                        &submesh_config.vertices,
                    )?;

                    let material = submesh_config.material;

                    materials.push(material.clone());

                    let submesh = SubmeshGPU::create(
                        indices_count,
                        index_id,
                        vertex_id,
                        material.id,
                        submesh_config.aabb,
                    );

                    self.resource_loader.load_buffer_at(
                        &self.buffer_manager.submesh_buffer.slice_at(SliceIndex::from(submesh_id)),
                        &[submesh],
                    )?;

                    index_id += indices_count;
                    vertex_id += vertices_count;
                    submesh_id += 1;
                }

                let mesh_gpu_data = MeshGPU::create(
                    first_submesh_id,
                    submesh_count,
                );

                self.resource_loader.load_buffer_at(
                    &self.buffer_manager.mesh_buffer.slice_at(SliceIndex::from(*id)),
                    &[mesh_gpu_data],
                )?;
                info!("Uploaded mesh: index: {}, data: {:?}", id, mesh_gpu_data);

                Ok(ManagedMesh {
                    first_index_id,
                    index_count,
                    first_vertex_id,
                    vertex_count,
                    first_submesh_id,
                    submesh_count,

                    skeleton,

                    materials,
                })
            }
        }
    }

    fn erase(&self, _id: &ResourceId) -> Result<()> {
        // self.resource_loader.load_buffer_at(
        //     &self.buffer_manager.mesh_buffer.slice_at(SliceIndex { value: *id }),
        //     &[self.default_mesh]
        // )?;

        Ok(())
    }

    fn destroy_resource(&self, _resource: Self::Output) -> Result<()> {
        Ok(())
    }
}
