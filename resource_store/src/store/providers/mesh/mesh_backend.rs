use gpu_data::MeshBoneGPU;
use gpu_data::SubmeshGPU;
use crate::store::mesh_table::geometry_range::GeometryRange;
use crate::store::mesh_table::mesh_table::MeshTable;
use std::slice::Iter;
use anyhow::{bail, Context, Result};
use rkyv::rancor::Error;
use rkyv::access;
use std::sync::Arc;
use resource_data::mesh_data::ArchivedMeshData;
use resource_data::submesh_data::ArchivedSubmeshData;
use gpu::ResourceTransfer;
use resource_residency::ResRef;
use resource_residency::ResourceBackend;
use resource_residency::ResourceProvider;
use index_allocator::ResourceId;
use crate::store::persistent::persistent_materials::PersistentMaterials;
use resource_reader::ResourceReader;
use crate::store::providers::material::material_backend::MaterialBackend;
use crate::store::providers::material::material_config::MaterialConfig;
use gpu::RangeAllocation;
use gpu_data::MeshVertexAttributeGPU;
use gpu_data::MeshVertexGPU;
use gpu_data::MeshVertexSkinGPU;
use crate::store::providers::mesh::extracted_submesh::ExtractedSubmesh;
use crate::store::providers::mesh::managed_mesh::ManagedMesh;
use crate::store::providers::mesh::mesh_backend_statistics::MeshBackendStatistics;
use crate::store::providers::mesh::mesh_config::MeshConfig;
use crate::store::providers::skeleton::skeleton_backend::SkeletonBackend;
use crate::store::providers::skeleton::skeleton_config::SkeletonConfig;

pub struct MeshBackend {
    resource_reader: Arc<dyn ResourceReader>,
    resource_transfer: Arc<ResourceTransfer>,

    material_provider: Arc<ResourceProvider<MaterialBackend>>,
    skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,

    mesh_table: Arc<MeshTable>,

    index: Arc<RangeAllocation<u32>>,
    mesh_vertex: Arc<RangeAllocation<MeshVertexGPU>>,
    mesh_vertex_attribute: Arc<RangeAllocation<MeshVertexAttributeGPU>>,
    mesh_vertex_skin: Arc<RangeAllocation<MeshVertexSkinGPU>>,
    mesh_bone: Arc<RangeAllocation<MeshBoneGPU>>,

    default_material: Arc<ResRef>,
}

impl MeshBackend {
    pub(crate) fn new(
        mesh_table: Arc<MeshTable>,
        index: Arc<RangeAllocation<u32>>,
        mesh_vertex: Arc<RangeAllocation<MeshVertexGPU>>,
        mesh_vertex_attribute: Arc<RangeAllocation<MeshVertexAttributeGPU>>,
        mesh_vertex_skin: Arc<RangeAllocation<MeshVertexSkinGPU>>,
        mesh_bone: Arc<RangeAllocation<MeshBoneGPU>>,
        persistent_materials: &PersistentMaterials,
        resource_reader: Arc<dyn ResourceReader>,
        resource_transfer: Arc<ResourceTransfer>,
        material_provider: Arc<ResourceProvider<MaterialBackend>>,
        skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
    ) -> Self {
        Self {
            resource_reader,
            resource_transfer,

            material_provider,
            skeleton_provider,

            mesh_table,

            index,
            mesh_vertex,
            mesh_vertex_attribute,
            mesh_vertex_skin,
            mesh_bone,
            
            default_material: persistent_materials.default.clone(),
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

    fn extract_archived_submeshes(
        &self,
        submeshes: Iter<'_, ArchivedSubmeshData>,
        skinned: bool,
    ) -> Result<Vec<ExtractedSubmesh>> {
        submeshes.map(|submesh_data| {
            let material = if let Some(resource_key) = submesh_data.material.as_ref() {
                self.material_provider.get_or_load(MaterialConfig::Alpaca {
                    resource_key: resource_key.value.to_string(),
                })?
            } else {
                self.default_material.clone()
            };

            Ok(ExtractedSubmesh::from_archived(submesh_data, skinned, material))
        }).collect::<Result<Vec<_>>>()
    }
}

impl ResourceBackend for MeshBackend {
    type Config = MeshConfig;
    type Output = ManagedMesh;
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

                let indices_allocation = self.index.allocator.allocate(index_count)
                    .with_context(|| format!("Failed to allocate {} indices", index_count))?;
                let vertices_allocation = self.mesh_vertex.allocator.allocate(vertex_count)
                    .with_context(|| format!("Failed to allocate {} vertices", vertex_count))?;
                let vertex_attributes_allocation = self.mesh_vertex_attribute.allocator.allocate(vertex_count)
                    .with_context(|| format!("Failed to allocate {} vertex attributes", vertex_count))?;
                let submeshes_allocation = self.mesh_table.submesh.allocator.allocate(submesh_count)
                    .with_context(|| format!("Failed to allocate {} submeshes", submesh_count))?;

                let vertex_skins_allocation = archived_mesh_data.skeleton.as_ref()
                    .map(|_| {
                        self.mesh_vertex_skin.allocator.allocate(vertex_count)
                            .with_context(|| format!("Failed to allocate {} vertex skins", vertex_count))
                    })
                    .transpose()?;

                let bones_allocation = archived_mesh_data.skeleton.as_ref()
                    .map(|_| {
                        let bone_count = archived_mesh_data.bones.len() as u32;

                        self.mesh_bone.allocator.allocate(bone_count)
                            .with_context(|| format!("Failed to allocate {} mesh bones", bone_count))
                    })
                    .transpose()?;

                let mut indices_offset = indices_allocation.offset;
                let mut vertices_offset = vertices_allocation.offset;
                let mut vertex_attributes_offset = vertex_attributes_allocation.offset;
                let mut vertex_skins_offset = vertex_skins_allocation.map(|allocation| allocation.offset);

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
                        self.index.slice(indices_offset, indices.len() as u32),
                        &indices,
                    )?;
                    self.resource_transfer.load_buffer_at(
                        self.mesh_vertex.slice(vertices_offset, vertices.len() as u32),
                        &vertices,
                    )?;
                    self.resource_transfer.load_buffer_at(
                        self.mesh_vertex_attribute.slice(vertex_attributes_offset, attributes.len() as u32),
                        &attributes,
                    )?;

                    if let Some(offset) = vertex_skins_offset {
                        self.resource_transfer.load_buffer_at(
                            self.mesh_vertex_skin.slice(offset, skins.len() as u32),
                            &skins,
                        )?;
                    }

                    materials.push(material.clone());

                    let submesh = SubmeshGPU::create(
                        indices.len() as u32,
                        indices_offset,
                        vertices_offset,
                        vertex_attributes_offset,
                        material.id.inner,
                        bounds,
                    );

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

                if let Some(allocation) = bones_allocation {
                    let bones = archived_mesh_data.bones.iter()
                        .map(|bone| MeshBoneGPU::create(
                            bone.inverse_bind_matrix.map(|column| column.map(|value| value.into())),
                            bone.bounds.map(|value| value.into()),
                        ))
                        .collect::<Vec<_>>();

                    self.resource_transfer.load_buffer_at(
                        self.mesh_bone.slice(allocation.offset, allocation.size),
                        &bones,
                    )?;
                }

                self.mesh_table.write(
                    *id,
                    submeshes_allocation,
                    &submeshes_gpu,
                    bones_allocation.map_or(0, |allocation| allocation.offset),
                    geometry_ranges,
                )?;

                Ok(ManagedMesh {
                    indices_allocation,
                    vertices_allocation,
                    vertex_attributes_allocation,
                    vertex_skins_allocation,
                    bones_allocation,
                    submeshes_allocation,

                    skeleton,

                    materials,
                })
            }
        }
    }

    fn erase(&self, id: &ResourceId) -> Result<()> {
        self.mesh_table.erase(*id)
    }

    fn statistics(&self) -> Self::Statistics {
        Self::Statistics {
            index: self.index.allocator.statistics(),
            vertex: self.mesh_vertex.allocator.statistics(),
            vertex_attribute: self.mesh_vertex_attribute.allocator.statistics(),
            vertex_skin: self.mesh_vertex_skin.allocator.statistics(),
            submesh: self.mesh_table.submesh.allocator.statistics(),
        }
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        self.index.allocator.release(resource.indices_allocation);

        self.mesh_vertex.allocator.release(resource.vertices_allocation);
        self.mesh_vertex_attribute.allocator.release(resource.vertex_attributes_allocation);

        if let Some(vertex_skins_allocation) = resource.vertex_skins_allocation {
            self.mesh_vertex_skin.allocator.release(vertex_skins_allocation);
        }
        if let Some(bones_allocation) = resource.bones_allocation {
            self.mesh_bone.allocator.release(bones_allocation);
        }
        self.mesh_table.submesh.allocator.release(resource.submeshes_allocation);

        Ok(())
    }

    fn destroy(self) -> Result<()> {
        Ok(())
    }
}
