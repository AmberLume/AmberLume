use std::slice::Iter;
use anyhow::Result;
use parking_lot::Mutex;
use rkyv::rancor::Error;
use rkyv::access;
use std::sync::Arc;
use tracing::info;
use crate::data::mesh_data::ArchivedMeshData;
use crate::data::submesh_data::ArchivedSubmeshData;
use gpu::SliceIndex;
use crate::limits::ResourceLimits;
use gpu::BufferInfo;
use gpu::ManagedBufferFactory;
use gpu::SliceBuffer;
use gpu::ResourceTransfer;
use crate::resources::alpaca_resource_reader::AlpacaResourceReader;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_backend::ResourceBackend;
use crate::resources::store::providers::resource_provider::{ResourceId, ResourceProvider};
use crate::resources::store::persistent::persistent_materials::PersistentMaterials;
use gpu::{Allocation, RangeAllocator};
use crate::resources::store::providers::material::material_backend::MaterialBackend;
use crate::resources::store::providers::material::material_config::MaterialConfig;
use crate::resources::store::providers::mesh::buffer::index_buffer::create_index_buffer;
use crate::resources::store::providers::mesh::buffer::mesh_buffer::{create_mesh_buffer, MeshGPU};
use crate::resources::store::providers::mesh::buffer::submesh_buffer::{create_submesh_buffer, SubmeshGPU};
use crate::resources::store::providers::mesh::buffer::vertex_buffer::{create_vertex_buffer, VertexGPU};
use crate::resources::store::providers::mesh::mesh_backend_statistics::MeshBackendStatistics;
use crate::resources::store::providers::mesh::mesh_load_observer::MeshLoadObserver;
use crate::resources::store::providers::mesh::mesh_config::{MeshConfig, SubmeshConfig};
use crate::resources::store::providers::skeleton::skeleton_backend::SkeletonBackend;
use crate::resources::store::providers::skeleton::skeleton_config::SkeletonConfig;

pub struct MeshBackend {
    alpaca_resource_reader: Arc<AlpacaResourceReader>,
    resource_transfer: Arc<ResourceTransfer>,
    
    material_provider: Arc<ResourceProvider<MaterialBackend>>,
    skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,

    submesh_allocator: RangeAllocator,
    index_allocator: RangeAllocator,
    vertex_allocator: RangeAllocator,
    
    pub mesh_buffer: SliceBuffer<MeshGPU>,
    pub submesh_buffer: SliceBuffer<SubmeshGPU>,
    pub index_buffer: SliceBuffer<u32>,
    pub vertex_buffer: SliceBuffer<VertexGPU>,

    default_material: Arc<ResRef>,

    load_observers: Mutex<Vec<Arc<dyn MeshLoadObserver>>>,
}

impl MeshBackend {
    pub fn new(
        limits: &ResourceLimits,
        buffer_factory: &ManagedBufferFactory,
        persistent_materials: &PersistentMaterials,
        alpaca_resource_reader: Arc<AlpacaResourceReader>,
        resource_transfer: Arc<ResourceTransfer>,
        material_provider: Arc<ResourceProvider<MaterialBackend>>,
        skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
        ray_tracing: bool,
    ) -> Result<Self> {
        let index_allocator = RangeAllocator::new(limits.max_indices);
        let vertex_allocator = RangeAllocator::new(limits.max_vertices);
        let submesh_allocator = RangeAllocator::new(limits.max_submeshes);

        let mesh_buffer = create_mesh_buffer(&buffer_factory, limits.max_meshes)?;
        let submesh_buffer = create_submesh_buffer(&buffer_factory, limits.max_submeshes)?;
        let index_buffer = create_index_buffer(&buffer_factory, limits.max_indices, ray_tracing)?;
        let vertex_buffer = create_vertex_buffer(&buffer_factory, limits.max_vertices, ray_tracing)?;

        Ok(Self {
            alpaca_resource_reader,
            resource_transfer,

            material_provider,
            skeleton_provider,

            submesh_allocator,
            index_allocator,
            vertex_allocator,

            mesh_buffer,
            submesh_buffer,
            index_buffer,
            vertex_buffer,
            
            default_material: persistent_materials.default.clone(),

            load_observers: Mutex::new(Vec::new()),
        })
    }

    pub fn subscribe(&self, observer: Arc<dyn MeshLoadObserver>) {
        self.load_observers.lock().push(observer);
    }

    fn notify_load(&self, mesh_id: ResourceId, mesh: &MeshGPU, submeshes: &[SubmeshGPU]) {
        let observers = self.load_observers.lock();

        for observer in observers.iter() {
            observer.on_load(mesh_id, mesh, submeshes);
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

            (indices, vertices, material, submesh_data.bounds.map(|v| v.into()))
        }).collect::<Vec<_>>()
    }
}

pub struct MeshHandle {
    pub indices_allocation: Allocation,
    pub vertices_allocation: Allocation,
    pub submeshes_allocation: Allocation,

    pub skeleton: Option<Arc<ResRef>>,

    pub materials: Vec<Arc<ResRef>>,
}

impl ResourceBackend for MeshBackend {
    type Config = MeshConfig;
    type Output = MeshHandle;
    type Statistics = MeshBackendStatistics;

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

                let indices_allocation = self.index_allocator.allocate(index_count).unwrap();
                let vertices_allocation = self.vertex_allocator.allocate(vertex_count).unwrap();
                let submeshes_allocation = self.submesh_allocator.allocate(submesh_count).unwrap();

                let mut indices_offset = indices_allocation.offset;
                let mut vertices_offset = vertices_allocation.offset;
                let mut submeshes_offset = submeshes_allocation.offset;

                let submeshes = self.extract_archived_submeshes(archived_mesh_data.submeshes.iter());

                let mut submeshes_gpu = Vec::new();

                for (indices, vertices, material, aabb) in submeshes {
                    self.resource_transfer.load_buffer_at(
                        &self.index_buffer.slice_at(SliceIndex::from(indices_offset)),
                        &indices,
                    )?;
                    self.resource_transfer.load_buffer_at(
                        &self.vertex_buffer.slice_at(SliceIndex::from(vertices_offset)),
                        &vertices,
                    )?;

                    materials.push(material.clone());

                    let submesh = SubmeshGPU::create(
                        indices.len() as u32,
                        indices_offset,
                        vertices_offset,
                        material.id.inner,
                        aabb,
                    );

                    self.resource_transfer.load_buffer_at(
                        &self.submesh_buffer.slice_at(SliceIndex::from(submeshes_offset)),
                        &[submesh],
                    )?;

                    submeshes_gpu.push(submesh);

                    indices_offset += indices.len() as u32;
                    vertices_offset += vertices.len() as u32;
                    submeshes_offset += 1;
                }

                let skeleton = archived_mesh_data.skeleton
                    .as_ref()
                    .map(|skeleton| {
                        self.skeleton_provider.get_or_load(SkeletonConfig::Alpaca {
                            resource_key: skeleton.value.to_string(),
                        })
                    });

                let mesh_gpu = MeshGPU::create(
                    submeshes_allocation.offset,
                    submeshes_allocation.size,
                );

                self.resource_transfer.load_buffer_at(
                    &self.mesh_buffer.slice_at(SliceIndex::from(id.inner)),
                    &[mesh_gpu],
                )?;
                info!("Uploaded mesh: index: {}, data: {:?}", id.inner, mesh_gpu);

                self.notify_load(*id, &mesh_gpu, &submeshes_gpu);

                Ok(MeshHandle {
                    indices_allocation,
                    vertices_allocation,
                    submeshes_allocation,

                    skeleton,

                    materials,
                })
            }
            Self::Config::InBuilt { submeshes, skeleton } => {
                let (index_count, vertex_count, submesh_count) = Self::count_config_index_vertex_submesh(&submeshes);

                let mut materials: Vec<Arc<ResRef>> = Vec::new();

                let indices_allocation = self.index_allocator.allocate(index_count).unwrap();
                let vertices_allocation = self.vertex_allocator.allocate(vertex_count).unwrap();
                let submeshes_allocation = self.submesh_allocator.allocate(submesh_count).unwrap();

                let mut indices_offset = indices_allocation.offset;
                let mut vertices_offset = vertices_allocation.offset;
                let mut submeshes_offset = submeshes_allocation.offset;

                let mut submeshes_gpu = Vec::new();

                for submesh_config in submeshes {
                    let indices_count = submesh_config.indices.len() as u32;
                    let vertices_count = submesh_config.vertices.len() as u32;

                    self.resource_transfer.load_buffer_at(
                        &self.index_buffer.slice_at(SliceIndex::from(indices_offset)),
                        &submesh_config.indices,
                    )?;
                    self.resource_transfer.load_buffer_at(
                        &self.vertex_buffer.slice_at(SliceIndex::from(vertices_offset)),
                        &submesh_config.vertices,
                    )?;

                    let material = submesh_config.material;

                    materials.push(material.clone());

                    let submesh = SubmeshGPU::create(
                        indices_count,
                        indices_offset,
                        vertices_offset,
                        material.id.inner,
                        submesh_config.aabb,
                    );

                    self.resource_transfer.load_buffer_at(
                        &self.submesh_buffer.slice_at(SliceIndex::from(submeshes_offset)),
                        &[submesh],
                    )?;

                    submeshes_gpu.push(submesh);

                    indices_offset += indices_count;
                    vertices_offset += vertices_count;
                    submeshes_offset += 1;
                }

                let mesh_gpu = MeshGPU::create(
                    submeshes_allocation.offset,
                    submeshes_allocation.size,
                );

                self.resource_transfer.load_buffer_at(
                    &self.mesh_buffer.slice_at(SliceIndex::from(id.inner)),
                    &[mesh_gpu],
                )?;
                info!("Uploaded mesh: index: {}, data: {:?}", id.inner, mesh_gpu);

                self.notify_load(*id, &mesh_gpu, &submeshes_gpu);

                Ok(MeshHandle {
                    indices_allocation,
                    vertices_allocation,
                    submeshes_allocation,

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

    fn statistics(&self) -> Self::Statistics {
        Self::Statistics {
            index: self.index_allocator.statistics(),
            vertex: self.vertex_allocator.statistics(),
            submesh: self.submesh_allocator.statistics(),
        }
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        self.index_allocator.release(resource.indices_allocation);
        self.vertex_allocator.release(resource.vertices_allocation);
        self.submesh_allocator.release(resource.submeshes_allocation);

        Ok(())
    }

    fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        buffer_factory.destroy_buffer(self.mesh_buffer.into_managed_buffer())?;
        buffer_factory.destroy_buffer(self.submesh_buffer.into_managed_buffer())?;
        buffer_factory.destroy_buffer(self.index_buffer.into_managed_buffer())?;
        buffer_factory.destroy_buffer(self.vertex_buffer.into_managed_buffer())?;
        
        Ok(())
    }
}
