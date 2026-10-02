use crate::store::providers::mesh::geometry_range::GeometryRange;
use crate::store::providers::mesh::managed_mesh::ManagedMesh;
use crate::store::providers::mesh::managed_mesh_instance::ManagedMeshInstance;
use crate::store::providers::mesh::managed_mesh_skeletal::ManagedMeshSkeletal;
use crate::store::providers::mesh::mesh_backend_statistics::MeshBackendStatistics;
use crate::store::providers::mesh::mesh_config::MeshConfig;
use crate::store::providers::mesh::submesh_config::SubmeshConfig;
use crate::store::vertex_allocation::VertexAllocation;
use anyhow::{bail, Context, Result};
use dashmap::DashMap;
use gpu::RangeAllocation;
use gpu::ResourceTransfer;
use gpu::SingleAllocation;
use gpu_data::MeshBindingGPU;
use gpu_data::MeshGPU;
use gpu_data::MeshVertexSkinGPU;
use gpu_data::SubmeshBoundsGPU;
use gpu_data::SubmeshGPU;
use gpu_data::VertexUvGPU;
use index_allocator::Allocation;
use index_allocator::ResourceId;
use resource_residency::ResourceBackend;
use std::sync::Arc;
use tracing::info;

pub struct MeshBackend {
    resource_transfer: Arc<ResourceTransfer>,

    mesh: Arc<SingleAllocation<MeshGPU>>,
    submesh: Arc<RangeAllocation<SubmeshGPU>>,

    index: Arc<RangeAllocation<u32>>,
    vertex: Arc<VertexAllocation>,
    vertex_uv: Arc<RangeAllocation<VertexUvGPU>>,
    submesh_bounds: Arc<RangeAllocation<SubmeshBoundsGPU>>,
    mesh_vertex_skin: Arc<RangeAllocation<MeshVertexSkinGPU>>,
    mesh_binding: Arc<RangeAllocation<MeshBindingGPU>>,

    meshes: DashMap<ResourceId, ManagedMesh>,
    instances: DashMap<ResourceId, ManagedMeshInstance>,
}

impl MeshBackend {
    pub(crate) fn new(
        resource_transfer: Arc<ResourceTransfer>,
        mesh: Arc<SingleAllocation<MeshGPU>>,
        submesh: Arc<RangeAllocation<SubmeshGPU>>,
        index: Arc<RangeAllocation<u32>>,
        vertex: Arc<VertexAllocation>,
        vertex_uv: Arc<RangeAllocation<VertexUvGPU>>,
        submesh_bounds: Arc<RangeAllocation<SubmeshBoundsGPU>>,
        mesh_vertex_skin: Arc<RangeAllocation<MeshVertexSkinGPU>>,
        mesh_binding: Arc<RangeAllocation<MeshBindingGPU>>,
    ) -> Self {
        Self {
            resource_transfer,

            mesh,
            submesh,

            index,
            vertex,
            vertex_uv,
            submesh_bounds,
            mesh_vertex_skin,
            mesh_binding,

            meshes: DashMap::new(),
            instances: DashMap::new(),
        }
    }

    pub fn with_mesh<R>(&self, id: ResourceId, action: impl FnOnce(&ManagedMesh) -> R) -> Option<R> {
        self.meshes.get(&id).map(|mesh| action(mesh.value()))
    }

    pub fn with_instance<R>(&self, id: ResourceId, action: impl FnOnce(&ManagedMeshInstance) -> R) -> Option<R> {
        self.instances.get(&id).map(|instance| action(instance.value()))
    }

    fn release_mesh(&self, mesh: ManagedMesh) {
        self.index.allocator.release(mesh.indices_allocation);
        self.vertex.allocator.release(mesh.vertices_allocation);
        self.vertex_uv.allocator.release(mesh.uvs_allocation);
        self.submesh.allocator.release(mesh.submeshes_allocation);
        self.submesh_bounds.allocator.release(mesh.bounds_allocation);

        if let Some(skeletal) = mesh.skeletal {
            self.mesh_vertex_skin.allocator.release(skeletal.vertex_skins_allocation);
            self.mesh_binding.allocator.release(skeletal.bindings_allocation);
        }
    }

    fn release_instance(&self, instance: ManagedMeshInstance) {
        self.vertex.allocator.release(instance.vertices_allocation);
        self.submesh.allocator.release(instance.submeshes_allocation);
        self.submesh_bounds.allocator.release(instance.bounds_allocation);
    }

    fn upload_mesh(
        &self,
        id: &ResourceId,
        mesh: &ManagedMesh,
        submeshes: &[SubmeshConfig],
        bindings: &[MeshBindingGPU],
    ) -> Result<()> {
        let mut vertex_skins_offset = mesh.skeletal.as_ref().map(|skeletal| skeletal.vertex_skins_allocation.offset);

        for (record, submesh) in mesh.submeshes.iter().zip(submeshes) {
            self.resource_transfer.load_buffer_at(
                self.index.slice(record.index_offset, submesh.indices.len() as u32),
                &submesh.indices,
            )?;
            self.resource_transfer.load_buffer_at(
                self.vertex.position_slice(record.vertex_offset, submesh.positions.len() as u32),
                &submesh.positions,
            )?;
            self.resource_transfer.load_buffer_at(
                self.vertex.normal_tangent_slice(record.vertex_offset, submesh.normal_tangents.len() as u32),
                &submesh.normal_tangents,
            )?;
            self.resource_transfer.load_buffer_at(
                self.vertex_uv.slice(record.uv_offset, submesh.uvs.len() as u32),
                &submesh.uvs,
            )?;
            self.resource_transfer.load_buffer_at(
                self.submesh_bounds.slice(record.bounds_index, 1),
                &[SubmeshBoundsGPU::create(submesh.bounds)],
            )?;

            if let Some(offset) = vertex_skins_offset {
                self.resource_transfer.load_buffer_at(
                    self.mesh_vertex_skin.slice(offset, submesh.skins.len() as u32),
                    &submesh.skins,
                )?;

                vertex_skins_offset = Some(offset + submesh.skins.len() as u32);
            }
        }

        if let Some(skeletal) = &mesh.skeletal {
            self.resource_transfer.load_buffer_at(
                self.mesh_binding.slice(skeletal.bindings_allocation.offset, skeletal.bindings_allocation.size),
                bindings,
            )?;
        }

        self.write_record(
            id,
            mesh.submeshes_allocation,
            &mesh.submeshes,
            mesh.skeletal.as_ref().map_or(0, |skeletal| skeletal.bindings_allocation.offset),
            mesh.vertex_slice_stride,
            mesh.vertex_slice_count,
        )
    }

    fn upload_instance(
        &self,
        id: &ResourceId,
        instance: &ManagedMeshInstance,
        binding_offset: u32,
        bounds: Option<[f32; 6]>,
    ) -> Result<()> {
        if let Some(bounds) = bounds {
            self.resource_transfer.load_buffer_at(
                self.submesh_bounds.slice(instance.bounds_allocation.offset, 1),
                &[SubmeshBoundsGPU::create(bounds)],
            )?;
        }

        self.write_record(
            id,
            instance.submeshes_allocation,
            &instance.submeshes,
            binding_offset,
            instance.vertex_slice_stride,
            instance.vertex_slice_count,
        )
    }

    fn write_record(
        &self,
        id: &ResourceId,
        submeshes_allocation: Allocation,
        submeshes: &[SubmeshGPU],
        binding_offset: u32,
        vertex_slice_stride: u32,
        vertex_slice_count: u32,
    ) -> Result<()> {
        let record = MeshGPU::create(
            submeshes_allocation.offset,
            submeshes_allocation.size,
            binding_offset,
            vertex_slice_stride,
            vertex_slice_count,
        );

        self.resource_transfer.load_buffer_at(
            self.submesh.slice(submeshes_allocation.offset, submeshes_allocation.size),
            submeshes,
        )?;

        self.resource_transfer.load_buffer_at(
            self.mesh.at(id.inner),
            &[record],
        )?;
        info!("Uploaded mesh: index: {}, data: {:?}", id.inner, record);

        Ok(())
    }
}

impl ResourceBackend for MeshBackend {
    type Config = MeshConfig;
    type Output = ();
    type Statistics = MeshBackendStatistics;

    fn reserve(&self, id: &ResourceId) -> Result<()> {
        self.erase(id)
    }

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output> {
        match config {
            MeshConfig::Data {
                submeshes,
                bindings,

                skeleton,
            } => {
                if submeshes.is_empty() {
                    bail!("Mesh has no submeshes");
                }

                let index_count = submeshes.iter().map(|submesh| submesh.indices.len() as u32).sum::<u32>();
                let vertex_count = submeshes.iter().map(|submesh| submesh.positions.len() as u32).sum::<u32>();
                let uv_count = submeshes.iter().map(|submesh| submesh.uvs.len() as u32).sum::<u32>();
                let vertex_skin_count = submeshes.iter().map(|submesh| submesh.skins.len() as u32).sum::<u32>();
                let binding_count = bindings.len() as u32;
                let submesh_count = submeshes.len() as u32;

                let indices = self.index.allocator.allocate(index_count);
                let vertices = self.vertex.allocator.allocate(vertex_count);
                let uvs = self.vertex_uv.allocator.allocate(uv_count);
                let vertex_skins = match skeleton {
                    Some(_) => self.mesh_vertex_skin.allocator.allocate(vertex_skin_count).map(Some),
                    None => Some(None),
                };
                let mesh_bindings = match skeleton {
                    Some(_) => self.mesh_binding.allocator.allocate(binding_count).map(Some),
                    None => Some(None),
                };
                let submesh_records = self.submesh.allocator.allocate(submesh_count);
                let bounds = self.submesh_bounds.allocator.allocate(submesh_count);

                let (
                    Some(indices_allocation),
                    Some(vertices_allocation),
                    Some(uvs_allocation),
                    Some(vertex_skins_allocation),
                    Some(bindings_allocation),
                    Some(submeshes_allocation),
                    Some(bounds_allocation),
                ) = (indices, vertices, uvs, vertex_skins, mesh_bindings, submesh_records, bounds) else {
                    if let Some(indices) = indices {
                        self.index.allocator.release(indices);
                    }
                    if let Some(vertices) = vertices {
                        self.vertex.allocator.release(vertices);
                    }
                    if let Some(uvs) = uvs {
                        self.vertex_uv.allocator.release(uvs);
                    }
                    if let Some(Some(vertex_skins)) = vertex_skins {
                        self.mesh_vertex_skin.allocator.release(vertex_skins);
                    }
                    if let Some(Some(mesh_bindings)) = mesh_bindings {
                        self.mesh_binding.allocator.release(mesh_bindings);
                    }
                    if let Some(submesh_records) = submesh_records {
                        self.submesh.allocator.release(submesh_records);
                    }
                    if let Some(bounds) = bounds {
                        self.submesh_bounds.allocator.release(bounds);
                    }

                    bail!(
                        "Failed to allocate mesh: {} indices, {} vertices, {} uvs, {} vertex skins, {} bindings, {} submeshes",
                        index_count, vertex_count, uv_count, vertex_skin_count, binding_count, submesh_count,
                    );
                };

                let mut records = Vec::with_capacity(submeshes.len());
                let mut geometry_ranges = Vec::with_capacity(submeshes.len());
                let mut materials = Vec::with_capacity(submeshes.len());

                let mut index_offset = indices_allocation.offset;
                let mut vertex_offset = vertices_allocation.offset;
                let mut uv_offset = uvs_allocation.offset;

                for (bounds_index, submesh) in (bounds_allocation.offset..).zip(&submeshes) {
                    let submesh_index_count = submesh.indices.len() as u32;
                    let submesh_vertex_count = submesh.positions.len() as u32;

                    records.push(SubmeshGPU::create(
                        submesh_index_count,
                        index_offset,
                        vertex_offset,
                        uv_offset,
                        submesh.material.id.inner,
                        bounds_index,
                    ));
                    geometry_ranges.push(GeometryRange {
                        index_count: submesh_index_count,
                        index_offset,
                        vertex_offset,
                        vertex_count: submesh_vertex_count,
                    });
                    materials.push(submesh.material.clone());

                    index_offset += submesh_index_count;
                    vertex_offset += submesh_vertex_count;
                    uv_offset += submesh.uvs.len() as u32;
                }

                let skeletal = skeleton
                    .zip(vertex_skins_allocation)
                    .zip(bindings_allocation)
                    .map(|((skeleton, vertex_skins_allocation), bindings_allocation)| ManagedMeshSkeletal {
                        vertex_skins_allocation,
                        bindings_allocation,

                        skeleton,
                    });

                let mesh = ManagedMesh {
                    indices_allocation,
                    vertices_allocation,
                    uvs_allocation,
                    submeshes_allocation,
                    bounds_allocation,

                    submeshes: records,
                    geometry_ranges,
                    vertex_slice_stride: vertex_count,
                    vertex_slice_count: 1,

                    skeletal,
                    materials,
                };

                if let Err(error) = self.upload_mesh(id, &mesh, &submeshes, &bindings) {
                    self.release_mesh(mesh);

                    return Err(error);
                }

                self.meshes.insert(*id, mesh);
            }
            MeshConfig::Instance {
                source,

                vertex_slice_count,
                bounds,
            } => {
                let source_mesh = self.meshes
                    .get(&source.id)
                    .with_context(|| format!("Mesh instance source {} is not resident", source.id.inner))?;

                let source_vertices = source_mesh.vertices_allocation;
                let vertex_count = source_vertices.size * vertex_slice_count;
                let submesh_count = source_mesh.submeshes.len() as u32;

                let vertices = self.vertex.allocator.allocate(vertex_count);
                let submesh_records = self.submesh.allocator.allocate(submesh_count);
                let instance_bounds = self.submesh_bounds.allocator.allocate(1);

                let (Some(vertices_allocation), Some(submeshes_allocation), Some(bounds_allocation)) =
                    (vertices, submesh_records, instance_bounds)
                else {
                    if let Some(vertices) = vertices {
                        self.vertex.allocator.release(vertices);
                    }
                    if let Some(submesh_records) = submesh_records {
                        self.submesh.allocator.release(submesh_records);
                    }
                    if let Some(instance_bounds) = instance_bounds {
                        self.submesh_bounds.allocator.release(instance_bounds);
                    }

                    bail!("Failed to allocate mesh instance: {} vertices, {} submeshes", vertex_count, submesh_count);
                };

                let records = source_mesh.submeshes
                    .iter()
                    .map(|submesh| {
                        SubmeshGPU::create(
                            submesh.index_count,
                            submesh.index_offset,
                            vertices_allocation.offset + (submesh.vertex_offset - source_vertices.offset),
                            submesh.uv_offset,
                            submesh.material_index,
                            bounds_allocation.offset,
                        )
                    })
                    .collect();
                let geometry_ranges = source_mesh.geometry_ranges
                    .iter()
                    .map(|geometry_range| GeometryRange {
                        vertex_offset: vertices_allocation.offset + (geometry_range.vertex_offset - source_vertices.offset),
                        ..*geometry_range
                    })
                    .collect();
                let binding_offset = source_mesh.skeletal.as_ref().map_or(0, |skeletal| skeletal.bindings_allocation.offset);

                drop(source_mesh);

                let instance = ManagedMeshInstance {
                    vertices_allocation,
                    submeshes_allocation,
                    bounds_allocation,

                    submeshes: records,
                    geometry_ranges,
                    vertex_slice_stride: source_vertices.size,
                    vertex_slice_count,

                    source,
                };

                if let Err(error) = self.upload_instance(id, &instance, binding_offset, bounds) {
                    self.release_instance(instance);

                    return Err(error);
                }

                self.instances.insert(*id, instance);
            }
        }

        Ok(())
    }

    fn erase(&self, id: &ResourceId) -> Result<()> {
        if let Some((_, mesh)) = self.meshes.remove(id) {
            self.release_mesh(mesh);
        }
        if let Some((_, instance)) = self.instances.remove(id) {
            self.release_instance(instance);
        }

        self.resource_transfer.load_buffer_at(
            self.mesh.at(id.inner),
            &[MeshGPU::create(0, 0, 0, 0, 0)],
        )
    }

    fn statistics(&self) -> Self::Statistics {
        Self::Statistics {
            index: self.index.allocator.statistics(),
            vertex: self.vertex.allocator.statistics(),
            vertex_uv: self.vertex_uv.allocator.statistics(),
            vertex_skin: self.mesh_vertex_skin.allocator.statistics(),
            submesh: self.submesh.allocator.statistics(),
            submesh_bounds: self.submesh_bounds.allocator.statistics(),
        }
    }

    fn destroy_resource(&self, _resource: Self::Output) -> Result<()> {
        Ok(())
    }

    fn destroy(self) -> Result<()> {
        Ok(())
    }
}
