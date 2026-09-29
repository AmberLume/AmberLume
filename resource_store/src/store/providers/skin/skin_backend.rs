use crate::store::blas_queue::blas_event::BlasEvent;
use crate::store::blas_queue::blas_queue::BlasQueue;
use crate::store::blas_queue::geometry_range::GeometryRange;
use crate::store::blas_queue::skin_geometry::SkinGeometry;
use crate::store::mesh_table::mesh_table::MeshTable;
use crate::store::providers::mesh::mesh_backend::MeshBackend;
use crate::store::providers::skin::frame_slice_index::FrameSliceIndex;
use crate::store::providers::skeleton::skeleton_backend::SkeletonBackend;
use crate::store::providers::skin::managed_skin::ManagedSkin;
use crate::store::providers::skin::skin_config::SkinConfig;
use crate::store::providers::skin::skin_source::SkinSource;
use crate::store::vertex_allocation::VertexAllocation;
use anyhow::{bail, Context, Result};
use gpu::RangeAllocation;
use gpu_data::SubmeshBoundsGPU;
use gpu_data::SubmeshGPU;
use index_allocator::Allocation;
use index_allocator::ResourceId;
use resource_residency::ResourceBackend;
use resource_residency::ResourceProvider;
use std::sync::Arc;

pub struct SkinBackend {
    mesh_provider: Arc<ResourceProvider<MeshBackend>>,
    skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,

    mesh_table: Arc<MeshTable>,
    blas_queue: Option<Arc<BlasQueue>>,

    vertex: Arc<VertexAllocation>,
    submesh_bounds: Arc<RangeAllocation<SubmeshBoundsGPU>>,

    slice_count: u32,
}

impl SkinBackend {
    pub(crate) fn new(
        mesh_table: Arc<MeshTable>,
        blas_queue: Option<Arc<BlasQueue>>,
        vertex: Arc<VertexAllocation>,
        submesh_bounds: Arc<RangeAllocation<SubmeshBoundsGPU>>,
        mesh_provider: Arc<ResourceProvider<MeshBackend>>,
        skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
        slice_count: u32,
    ) -> Result<Self> {
        if slice_count < 2 {
            bail!("Skin slice count must be at least 2, got {}", slice_count);
        }

        Ok(Self {
            mesh_provider,
            skeleton_provider,

            mesh_table,
            blas_queue,

            vertex,
            submesh_bounds,

            slice_count,
        })
    }

    fn write_slices(&self, skin: &ManagedSkin, source: &SkinSource) -> Result<()> {
        let slice_count = self.slice_count;
        let submesh_count = source.submeshes.len() as u32;

        for frame in 0..slice_count as u64 {
            let frame_slice_index = FrameSliceIndex::create(frame, slice_count);

            let vertex_offset = skin.vertices_allocation.offset + frame_slice_index.current * skin.vertex_count;
            let previous_vertex_offset = skin.vertices_allocation.offset + frame_slice_index.previous * skin.vertex_count;

            let submeshes = source.submeshes
                .iter()
                .map(|submesh| {
                    let local_vertex_offset = submesh.vertex_offset - skin.source_vertex_offset;

                    SubmeshGPU::create(
                        submesh.index_count,
                        submesh.index_offset,
                        vertex_offset + local_vertex_offset,
                        previous_vertex_offset + local_vertex_offset,
                        submesh.uv_offset,
                        submesh.material_index,
                        skin.bounds_allocation.offset,
                    )
                })
                .collect::<Vec<_>>();

            self.mesh_table.write(
                skin.mesh_ids[frame_slice_index.current as usize],
                Allocation {
                    offset: skin.submeshes_allocation.offset + frame_slice_index.current * submesh_count,
                    size: submesh_count,
                },
                &submeshes,
                source.bone_offset,
            )?;
        }

        Ok(())
    }
}

impl ResourceBackend for SkinBackend {
    type Config = SkinConfig;
    type Output = ManagedSkin;
    type Statistics = ();

    fn create(&self, id: &ResourceId, config: Self::Config) -> Result<Self::Output> {
        let SkinConfig {
            owner: _,

            mesh,
            skeleton,
        } = config;

        let bone_count = self.skeleton_provider
            .with_resource(skeleton.id, |skeleton| skeleton.bones_allocation.size)
            .context("Skin skeleton is not resident")?;

        let source = self.mesh_provider
            .with_resource(mesh.id, |mesh| {
                mesh.vertex_skins_allocation.map(|vertex_skins_allocation| SkinSource {
                    vertices_allocation: mesh.vertices_allocation,
                    vertex_skins_allocation,
                    bone_offset: mesh.bones_allocation.map_or(0, |bones_allocation| bones_allocation.offset),

                    submeshes: mesh.submeshes.clone(),
                    geometry_ranges: mesh.geometry_ranges.clone(),
                })
            })
            .context("Skin mesh is not resident")?
            .context("Skin mesh has no vertex skins")?;

        let slice_count = self.slice_count;
        let vertex_count = source.vertices_allocation.size;
        let submesh_count = source.submeshes.len() as u32;

        let mut mesh_ids = Vec::with_capacity(slice_count as usize);

        for _ in 0..slice_count {
            let Some(mesh_id) = self.mesh_table.mesh.allocator.acquire() else {
                break;
            };

            mesh_ids.push(mesh_id);
        }

        let vertices_allocation = self.vertex.allocator.allocate(slice_count * vertex_count);
        let submeshes_allocation = self.mesh_table.submesh.allocator.allocate(slice_count * submesh_count);
        let bounds_allocation = self.submesh_bounds.allocator.allocate(1);

        let (true, Some(vertices_allocation), Some(submeshes_allocation), Some(bounds_allocation)) = (
            mesh_ids.len() == slice_count as usize,
            vertices_allocation,
            submeshes_allocation,
            bounds_allocation,
        ) else {
            for mesh_id in mesh_ids {
                self.mesh_table.mesh.allocator.release(mesh_id);
            }
            if let Some(vertices_allocation) = vertices_allocation {
                self.vertex.allocator.release(vertices_allocation);
            }
            if let Some(submeshes_allocation) = submeshes_allocation {
                self.mesh_table.submesh.allocator.release(submeshes_allocation);
            }
            if let Some(bounds_allocation) = bounds_allocation {
                self.submesh_bounds.allocator.release(bounds_allocation);
            }

            bail!("Resource buffers are full");
        };

        let skin = ManagedSkin {
            mesh_ids,
            vertices_allocation,
            submeshes_allocation,
            bounds_allocation,

            vertex_count,
            source_vertex_offset: source.vertices_allocation.offset,
            source_skin_offset: source.vertex_skins_allocation.offset,
            bone_count,

            mesh,
            skeleton,
        };

        if let Err(error) = self.write_slices(&skin, &source) {
            self.destroy_resource(skin)?;

            return Err(error);
        }

        if let Some(blas_queue) = &self.blas_queue {
            blas_queue.push(BlasEvent::SkinLoaded {
                skin_id: *id,
                geometry: SkinGeometry {
                    geometry_ranges: source.geometry_ranges
                        .iter()
                        .map(|geometry_range| GeometryRange {
                            vertex_offset: geometry_range.vertex_offset - skin.source_vertex_offset,
                            ..*geometry_range
                        })
                        .collect(),

                    vertex_offset: skin.vertices_allocation.offset,
                    vertex_count: skin.vertex_count,
                },
            });
        }

        Ok(skin)
    }

    fn erase(&self, id: &ResourceId) -> Result<()> {
        if let Some(blas_queue) = &self.blas_queue {
            blas_queue.push(BlasEvent::SkinUnloaded { skin_id: *id });
        }

        Ok(())
    }

    fn statistics(&self) -> Self::Statistics {
        ()
    }

    fn destroy_resource(&self, skin: Self::Output) -> Result<()> {
        let erased = skin.mesh_ids
            .iter()
            .try_for_each(|mesh_id| self.mesh_table.erase(*mesh_id));

        self.mesh_table.submesh.allocator.release(skin.submeshes_allocation);
        self.vertex.allocator.release(skin.vertices_allocation);
        self.submesh_bounds.allocator.release(skin.bounds_allocation);

        for mesh_id in skin.mesh_ids {
            self.mesh_table.mesh.allocator.release(mesh_id);
        }

        erased
    }
}
