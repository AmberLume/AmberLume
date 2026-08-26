use gpu_data::MeshGPU;
use gpu_data::SubmeshGPU;
use crate::store::providers::mesh::geometry_changes::GeometryChanges;
use crate::store::providers::mesh::geometry_range::GeometryRange;
use crate::store::providers::mesh::loaded_geometry::LoadedGeometry;
use gpu_data::MeshVertexAttributeGPU;
use gpu_data::MeshVertexGPU;
use gpu_data::MeshVertexSkinGPU;
use std::collections::HashMap;
use std::slice::Iter;
use anyhow::{bail, Context, Result};
use parking_lot::Mutex;
use rkyv::rancor::Error;
use rkyv::access;
use std::mem::take;
use std::sync::Arc;
use tracing::info;
use resource_data::mesh_data::ArchivedMeshData;
use resource_data::submesh_data::ArchivedSubmeshData;
use index_allocator::SliceIndex;
use index_allocator::ResourceLimits;
use gpu::BufferArray;
use gpu::ResourceTransfer;
use resource_residency::ResRef;
use resource_residency::ResourceBackend;
use resource_residency::ResourceHash;
use resource_residency::ResourceProvider;
use index_allocator::ResourceId;
use crate::store::persistent::persistent_materials::PersistentMaterials;
use index_allocator::Allocation;
use index_allocator::RangeAllocator;
use resource_reader::ResourceReader;
use crate::store::providers::material::material_backend::MaterialBackend;
use crate::store::providers::material::material_config::MaterialConfig;
use crate::store::providers::mesh::buffer::mesh_vertex_attribute_buffer::mesh_vertex_attribute_from_archived;
use crate::store::providers::mesh::buffer::mesh_vertex_buffer::mesh_vertex_from_archived;
use crate::store::providers::mesh::buffer::mesh_vertex_skin_buffer::mesh_vertex_skin_from_archived;
use crate::store::geometry::mesh_regions::MeshRegions;
use crate::store::providers::mesh::extracted_submesh::ExtractedSubmesh;
use crate::store::providers::mesh::mesh_backend_statistics::MeshBackendStatistics;
use crate::store::providers::mesh::shared_index_range::SharedIndexRange;
use crate::store::providers::mesh::mesh_config::{MeshConfig, SubmeshConfig};
use crate::store::providers::skeleton::skeleton_backend::SkeletonBackend;
use crate::store::providers::skeleton::skeleton_config::SkeletonConfig;

pub struct MeshBackend {
    resource_reader: Arc<dyn ResourceReader>,
    resource_transfer: Arc<ResourceTransfer>,

    material_provider: Arc<ResourceProvider<MaterialBackend>>,
    skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,

    index_allocator: RangeAllocator,
    pub(crate) index_buffer: BufferArray<u32>,

    vertex_allocator: RangeAllocator,
    pub(crate) vertex_buffer: BufferArray<MeshVertexGPU>,

    pub(crate) mesh_buffer: BufferArray<MeshGPU>,

    submesh_allocator: RangeAllocator,
    pub(crate) submesh_buffer: BufferArray<SubmeshGPU>,

    vertex_attribute_allocator: RangeAllocator,
    pub(crate) vertex_attribute_buffer: BufferArray<MeshVertexAttributeGPU>,

    vertex_skin_allocator: RangeAllocator,
    pub(crate) vertex_skin_buffer: BufferArray<MeshVertexSkinGPU>,

    default_material: Arc<ResRef>,

    shared_indices: Mutex<HashMap<ResourceHash, SharedIndexRange>>,

    geometry_changes: Mutex<GeometryChanges>,
}

impl MeshBackend {
    pub(crate) fn new(
        limits: &ResourceLimits,
        regions: MeshRegions,
        persistent_materials: &PersistentMaterials,
        resource_reader: Arc<dyn ResourceReader>,
        resource_transfer: Arc<ResourceTransfer>,
        material_provider: Arc<ResourceProvider<MaterialBackend>>,
        skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
    ) -> Result<Self> {
        let index_allocator = RangeAllocator::new(limits.max_indices);
        let vertex_allocator = RangeAllocator::new(limits.max_vertices);
        let submesh_allocator = RangeAllocator::new(limits.max_submeshes);
        let vertex_attribute_allocator = RangeAllocator::new(limits.max_vertex_attributes);
        let vertex_skin_allocator = RangeAllocator::new(limits.max_vertex_skins);

        let MeshRegions {
            index: index_buffer,
            mesh: mesh_buffer,
            submesh: submesh_buffer,
            vertex: vertex_buffer,
            vertex_attribute: vertex_attribute_buffer,
            vertex_skin: vertex_skin_buffer,
        } = regions;

        Ok(Self {
            resource_reader,
            resource_transfer,

            material_provider,
            skeleton_provider,

            index_allocator,
            index_buffer,

            mesh_buffer,

            submesh_allocator,
            submesh_buffer,

            vertex_allocator,
            vertex_buffer,

            vertex_attribute_allocator,
            vertex_attribute_buffer,

            vertex_skin_allocator,
            vertex_skin_buffer,
            
            default_material: persistent_materials.default.clone(),

            shared_indices: Mutex::new(HashMap::new()),

            geometry_changes: Mutex::new(GeometryChanges::default()),
        })
    }

    fn acquire_shared_indices(&self, hash: ResourceHash, indices: &[u32]) -> Result<Allocation> {
        let mut shared_indices = self.shared_indices.lock();

        if let Some(shared_index_range) = shared_indices.get_mut(&hash) {
            shared_index_range.users += 1;

            return Ok(shared_index_range.allocation);
        }

        let allocation = self.index_allocator.allocate(indices.len() as u32)
            .with_context(|| format!("Failed to reserve {} shared indices", indices.len()))?;

        self.resource_transfer.load_buffer_at(
            self.index_buffer.slice(SliceIndex::from(allocation.offset), indices.len() as u32),
            indices,
        )?;

        shared_indices.insert(hash, SharedIndexRange { allocation, users: 1 });

        Ok(allocation)
    }

    fn release_shared_indices(&self, hash: ResourceHash) {
        let mut shared_indices = self.shared_indices.lock();

        let Some(shared_index_range) = shared_indices.get_mut(&hash) else {
            return;
        };

        shared_index_range.users -= 1;

        if shared_index_range.users > 0 {
            return;
        }

        self.index_allocator.release(shared_index_range.allocation);

        shared_indices.remove(&hash);
    }

    pub fn take_geometry_changes(&self) -> GeometryChanges {
        take(&mut *self.geometry_changes.lock())
    }

    fn record_loaded(&self, mesh_id: ResourceId, ranges: Vec<GeometryRange>) {
        self.geometry_changes
            .lock()
            .loaded
            .push(LoadedGeometry { mesh_id, ranges });
    }

    fn record_unloaded(&self, mesh_id: ResourceId) {
        self.geometry_changes.lock().unloaded.push(mesh_id);
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

    fn extract_archived_submeshes(
        &self,
        submeshes: Iter<'_, ArchivedSubmeshData>,
        skinned: bool,
    ) -> Result<Vec<ExtractedSubmesh>> {
        submeshes.map(|submesh_data| {
            let indices = submesh_data.indices.iter()
                .map(|v| v.to_native())
                .collect::<Vec<_>>();
            let vertices = submesh_data.positions.iter().enumerate().map(|(index, _)| {
                mesh_vertex_from_archived(&submesh_data, index)
            }).collect::<Vec<_>>();
            let attributes = submesh_data.positions.iter().enumerate().map(|(index, _)| {
                mesh_vertex_attribute_from_archived(&submesh_data, index)
            }).collect::<Vec<_>>();
            let skins = if skinned {
                submesh_data.positions.iter().enumerate().map(|(index, _)| {
                    mesh_vertex_skin_from_archived(&submesh_data, index)
                }).collect::<Vec<_>>()
            } else {
                Vec::new()
            };

            let material = if let Some(resource_key) = submesh_data.material.as_ref() {
                self.material_provider.get_or_load(MaterialConfig::Alpaca {
                    resource_key: resource_key.value.to_string(),
                })?
            } else {
                self.default_material.clone()
            };

            Ok(ExtractedSubmesh {
                indices,

                vertices,
                attributes,
                skins,

                material,
                bounds: submesh_data.bounds.map(|v| v.into()),
            })
        }).collect::<Result<Vec<_>>>()
    }
}

pub struct MeshHandle {
    pub indices_allocation: Allocation,
    pub shared_indices: Option<ResourceHash>,
    pub vertices_allocation: Allocation,
    pub vertex_attributes_allocation: Allocation,
    pub vertex_skins_allocation: Option<Allocation>,
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
                let mesh_bytes = self.resource_reader.get_resource(&resource_key)?;
                let archived_mesh_data = access::<ArchivedMeshData, Error>(&mesh_bytes)?;

                let mut materials: Vec<Arc<ResRef>> = Vec::new();

                let (index_count, vertex_count, submesh_count) = Self::count_archived_index_vertex_submesh(&archived_mesh_data);

                let indices_allocation = self.index_allocator.allocate(index_count)
                    .with_context(|| format!("Failed to allocate {} indices", index_count))?;
                let vertices_allocation = self.vertex_allocator.allocate(vertex_count)
                    .with_context(|| format!("Failed to allocate {} vertices", vertex_count))?;
                let vertex_attributes_allocation = self.vertex_attribute_allocator.allocate(vertex_count)
                    .with_context(|| format!("Failed to allocate {} vertex attributes", vertex_count))?;
                let submeshes_allocation = self.submesh_allocator.allocate(submesh_count)
                    .with_context(|| format!("Failed to allocate {} submeshes", submesh_count))?;

                let vertex_skins_allocation = archived_mesh_data.skeleton.as_ref()
                    .map(|_| {
                        self.vertex_skin_allocator.allocate(vertex_count)
                            .with_context(|| format!("Failed to allocate {} vertex skins", vertex_count))
                    })
                    .transpose()?;

                let mut indices_offset = indices_allocation.offset;
                let mut vertices_offset = vertices_allocation.offset;
                let mut vertex_attributes_offset = vertex_attributes_allocation.offset;
                let mut vertex_skins_offset = vertex_skins_allocation.map(|allocation| allocation.offset);
                let mut submeshes_offset = submeshes_allocation.offset;

                let submeshes = self.extract_archived_submeshes(
                    archived_mesh_data.submeshes.iter(),
                    vertex_skins_allocation.is_some(),
                )?;

                let mut submeshes_gpu = Vec::new();
                let mut geometry_ranges = Vec::new();

                for extracted_submesh in submeshes {
                    let ExtractedSubmesh {
                        indices,
                        vertices,
                        attributes,
                        skins,
                        material,
                        bounds,
                    } = extracted_submesh;

                    self.resource_transfer.load_buffer_at(
                        self.index_buffer.slice(SliceIndex::from(indices_offset), indices.len() as u32),
                        &indices,
                    )?;
                    self.resource_transfer.load_buffer_at(
                        self.vertex_buffer.slice(SliceIndex::from(vertices_offset), vertices.len() as u32),
                        &vertices,
                    )?;
                    self.resource_transfer.load_buffer_at(
                        self.vertex_attribute_buffer.slice(SliceIndex::from(vertex_attributes_offset), attributes.len() as u32),
                        &attributes,
                    )?;

                    if let Some(offset) = vertex_skins_offset {
                        self.resource_transfer.load_buffer_at(
                            self.vertex_skin_buffer.slice(SliceIndex::from(offset), skins.len() as u32),
                            &skins,
                        )?;
                    }

                    materials.push(material.clone());

                    let submesh = SubmeshGPU::create(
                        indices.len() as u32,
                        indices_offset,
                        vertices_offset,
                        vertex_attributes_offset,
                        vertex_skins_offset.unwrap_or(0),
                        material.id.inner,
                        bounds,
                    );

                    self.resource_transfer.load_buffer_at(
                        self.submesh_buffer.at(SliceIndex::from(submeshes_offset)),
                        &[submesh],
                    )?;

                    submeshes_gpu.push(submesh);
                    geometry_ranges.push(GeometryRange {
                        index_count: indices.len() as u32,
                        index_offset: indices_offset,
                        vertex_offset: vertices_offset,
                        vertex_count: vertices.len() as u32,
                    });

                    indices_offset += indices.len() as u32;
                    vertices_offset += vertices.len() as u32;
                    vertex_attributes_offset += attributes.len() as u32;
                    vertex_skins_offset = vertex_skins_offset.map(|offset| offset + skins.len() as u32);
                    submeshes_offset += 1;
                }

                if submeshes_gpu.is_empty() {
                    bail!("Mesh has no submeshes");
                }

                let skeleton = archived_mesh_data.skeleton
                    .as_ref()
                    .map(|skeleton| {
                        self.skeleton_provider.get_or_load(SkeletonConfig::Alpaca {
                            resource_key: skeleton.value.to_string(),
                        })
                    })
                    .transpose()?;

                let mesh_gpu = MeshGPU::create(
                    submeshes_allocation.offset,
                    submeshes_allocation.size,
                );

                self.resource_transfer.load_buffer_at(
                    self.mesh_buffer.at(SliceIndex::from(id.inner)),
                    &[mesh_gpu],
                )?;
                info!("Uploaded mesh: index: {}, data: {:?}", id.inner, mesh_gpu);

                self.record_loaded(*id, geometry_ranges);

                Ok(MeshHandle {
                    indices_allocation,
                    shared_indices: None,
                    vertices_allocation,
                    vertex_attributes_allocation,
                    vertex_skins_allocation,
                    submeshes_allocation,

                    skeleton,

                    materials,
                })
            }
            Self::Config::InBuilt { submeshes, skeleton } => {
                let (index_count, vertex_count, submesh_count) = Self::count_config_index_vertex_submesh(&submeshes);

                let mut materials: Vec<Arc<ResRef>> = Vec::new();

                let indices_allocation = self.index_allocator.allocate(index_count)
                    .with_context(|| format!("Failed to allocate {} indices", index_count))?;
                let vertices_allocation = self.vertex_allocator.allocate(vertex_count)
                    .with_context(|| format!("Failed to allocate {} vertices", vertex_count))?;
                let vertex_attributes_allocation = self.vertex_attribute_allocator.allocate(vertex_count)
                    .with_context(|| format!("Failed to allocate {} vertex attributes", vertex_count))?;
                let submeshes_allocation = self.submesh_allocator.allocate(submesh_count)
                    .with_context(|| format!("Failed to allocate {} submeshes", submesh_count))?;

                let vertex_skins_allocation = None;

                let mut indices_offset = indices_allocation.offset;
                let mut vertices_offset = vertices_allocation.offset;
                let mut vertex_attributes_offset = vertex_attributes_allocation.offset;
                let mut submeshes_offset = submeshes_allocation.offset;

                let mut submeshes_gpu = Vec::new();
                let mut geometry_ranges = Vec::new();

                for submesh_config in submeshes {
                    let indices_count = submesh_config.indices.len() as u32;
                    let vertices_count = submesh_config.vertices.len() as u32;

                    self.resource_transfer.load_buffer_at(
                        self.index_buffer.slice(SliceIndex::from(indices_offset), submesh_config.indices.len() as u32),
                        &submesh_config.indices,
                    )?;
                    self.resource_transfer.load_buffer_at(
                        self.vertex_buffer.slice(SliceIndex::from(vertices_offset), submesh_config.vertices.len() as u32),
                        &submesh_config.vertices,
                    )?;
                    self.resource_transfer.load_buffer_at(
                        self.vertex_attribute_buffer.slice(SliceIndex::from(vertex_attributes_offset), submesh_config.attributes.len() as u32),
                        &submesh_config.attributes,
                    )?;

                    let material = submesh_config.material;

                    materials.push(material.clone());

                    let submesh = SubmeshGPU::create(
                        indices_count,
                        indices_offset,
                        vertices_offset,
                        vertex_attributes_offset,
                        0,
                        material.id.inner,
                        submesh_config.aabb,
                    );

                    self.resource_transfer.load_buffer_at(
                        self.submesh_buffer.at(SliceIndex::from(submeshes_offset)),
                        &[submesh],
                    )?;

                    submeshes_gpu.push(submesh);
                    geometry_ranges.push(GeometryRange {
                        index_count: indices_count,
                        index_offset: indices_offset,
                        vertex_offset: vertices_offset,
                        vertex_count: vertices_count,
                    });

                    indices_offset += indices_count;
                    vertices_offset += vertices_count;
                    vertex_attributes_offset += vertices_count;
                    submeshes_offset += 1;
                }

                if submeshes_gpu.is_empty() {
                    bail!("Mesh has no submeshes");
                }

                let mesh_gpu = MeshGPU::create(
                    submeshes_allocation.offset,
                    submeshes_allocation.size,
                );

                self.resource_transfer.load_buffer_at(
                    self.mesh_buffer.at(SliceIndex::from(id.inner)),
                    &[mesh_gpu],
                )?;
                info!("Uploaded mesh: index: {}, data: {:?}", id.inner, mesh_gpu);

                self.record_loaded(*id, geometry_ranges);

                Ok(MeshHandle {
                    indices_allocation,
                    shared_indices: None,
                    vertices_allocation,
                    vertex_attributes_allocation,
                    vertex_skins_allocation,
                    submeshes_allocation,

                    skeleton,

                    materials,
                })
            }
            Self::Config::Reserved {
                key: _,
                indices,
                vertex_count,
                material,
                bounds,
            } => {
                let shared_indices = ResourceHash::of(&indices);
                let indices_allocation = self.acquire_shared_indices(shared_indices, &indices)?;

                let vertices_allocation = self.vertex_allocator.allocate(vertex_count)
                    .with_context(|| format!("Failed to reserve {} vertices", vertex_count))?;
                let vertex_attributes_allocation = self.vertex_attribute_allocator.allocate(vertex_count)
                    .with_context(|| format!("Failed to reserve {} vertex attributes", vertex_count))?;
                let submeshes_allocation = self.submesh_allocator.allocate(1)
                    .context("Failed to reserve a submesh")?;

                let submesh = SubmeshGPU::create(
                    indices_allocation.size,
                    indices_allocation.offset,
                    vertices_allocation.offset,
                    vertex_attributes_allocation.offset,
                    0,
                    material.id.inner,
                    bounds,
                );

                self.resource_transfer.load_buffer_at(
                    self.submesh_buffer.at(SliceIndex::from(submeshes_allocation.offset)),
                    &[submesh],
                )?;

                let mesh_gpu = MeshGPU::create(
                    submeshes_allocation.offset,
                    submeshes_allocation.size,
                );

                self.resource_transfer.load_buffer_at(
                    self.mesh_buffer.at(SliceIndex::from(id.inner)),
                    &[mesh_gpu],
                )?;
                info!("Reserved mesh: index: {}, data: {:?}", id.inner, mesh_gpu);

                self.record_loaded(*id, vec![GeometryRange {
                    index_count: indices_allocation.size,
                    index_offset: indices_allocation.offset,
                    vertex_offset: vertices_allocation.offset,
                    vertex_count,
                }]);

                Ok(MeshHandle {
                    indices_allocation,
                    shared_indices: Some(shared_indices),
                    vertices_allocation,
                    vertex_attributes_allocation,
                    vertex_skins_allocation: None,
                    submeshes_allocation,

                    skeleton: None,

                    materials: vec![material],
                })
            }
        }
    }

    fn erase(&self, id: &ResourceId) -> Result<()> {
        self.record_unloaded(*id);

        self.resource_transfer.load_buffer_at(
            self.mesh_buffer.at(SliceIndex::from(id.inner)),
            &[MeshGPU::create(0, 0)],
        )?;

        Ok(())
    }

    fn statistics(&self) -> Self::Statistics {
        Self::Statistics {
            index: self.index_allocator.statistics(),
            vertex: self.vertex_allocator.statistics(),
            vertex_attribute: self.vertex_attribute_allocator.statistics(),
            vertex_skin: self.vertex_skin_allocator.statistics(),
            submesh: self.submesh_allocator.statistics(),
        }
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        match resource.shared_indices {
            Some(shared_indices) => self.release_shared_indices(shared_indices),
            None => self.index_allocator.release(resource.indices_allocation),
        }

        self.vertex_allocator.release(resource.vertices_allocation);
        self.vertex_attribute_allocator.release(resource.vertex_attributes_allocation);

        if let Some(vertex_skins_allocation) = resource.vertex_skins_allocation {
            self.vertex_skin_allocator.release(vertex_skins_allocation);
        }
        self.submesh_allocator.release(resource.submeshes_allocation);

        Ok(())
    }

    fn destroy(self) -> Result<()> {
        Ok(())
    }
}
