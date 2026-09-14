use anyhow::{Context, Result};
use ash::vk::{
    AccelerationStructureBuildRangeInfoKHR, AccessFlags, BuildAccelerationStructureModeKHR,
    DeviceOrHostAddressKHR, DeviceSize, PipelineStageFlags,
};
use gpu::GpuSize;
use gpu::ResourceFactories;
use gpu_data::MeshVertexGPU;
use index_allocator::ResourceId;
use ray_tracing::align_up;
use ray_tracing::skinned_blas_build_geometry_info;
use ray_tracing::SkinnedBlasEntry;
use ray_tracing::BLAS;
use render_graph::DataResourceScope;
use render_graph::FrameContext;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::PrepareScopes;
use render_graph::RecordScopes;
use render_graph::VirtualAccelerationStructure;
use render_graph::VirtualBuffer;
use render_graph::VirtualData;
use render_snapshot::RenderSnapshot;
use resource_residency::ResourceProvider;
use resource_store::GeometryRange;
use resource_store::MeshBackend;
use std::collections::HashSet;
use std::mem::size_of;
use std::sync::Arc;
use crate::render::pass::skinned_blas::skinned_blas_build::SkinnedBLASBuild;

pub struct SkinnedBLASPass {
    blas_state: VirtualData<Arc<BLAS>>,
    render_snapshot: VirtualData<RenderSnapshot>,

    blas: VirtualAccelerationStructure,
    blas_addresses: VirtualBuffer,
    scratch: VirtualBuffer,
    skin_cache_vertex: VirtualBuffer,
    index_buffer: VirtualBuffer,

    mesh_provider: Arc<ResourceProvider<MeshBackend>>,
}

impl SkinnedBLASPass {
    pub fn create(
        blas_state: VirtualData<Arc<BLAS>>,
        render_snapshot: VirtualData<RenderSnapshot>,
        blas: VirtualAccelerationStructure,
        blas_addresses: VirtualBuffer,
        scratch: VirtualBuffer,
        skin_cache_vertex: VirtualBuffer,
        index_buffer: VirtualBuffer,
        mesh_provider: Arc<ResourceProvider<MeshBackend>>,
    ) -> Self {
        Self {
            blas_state,
            render_snapshot,

            blas,
            blas_addresses,
            scratch,
            skin_cache_vertex,
            index_buffer,

            mesh_provider,
        }
    }
}

pub struct SkinnedBLASPassData {
    blas: Arc<BLAS>,
    builds: Vec<SkinnedBLASBuild>,
}

impl Pass for SkinnedBLASPass {
    type PassData = SkinnedBLASPassData;

    fn name(&self) -> String {
        String::from("skinned_blas")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.blas_state)
            .consume(self.render_snapshot)
            .write_acceleration_structure(
                self.blas,
                AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            )
            .read_buffer(
                self.skin_cache_vertex,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            )
            .read_buffer(
                self.index_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            )
            .write_buffer(
                self.blas_addresses,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            )
            .write_buffer(
                self.scratch,
                AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            );
    }

    fn prepare_data(
        &self,
        scopes: &mut PrepareScopes,
        frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let blas = scopes.data.get(self.blas_state).clone();
        let render_snapshot = scopes.data.get(self.render_snapshot);

        let skin_cache_vertex = scopes.buffer.get_physical_buffer(self.skin_cache_vertex);
        let mesh_addresses = blas.addresses();

        let alignment = self.scratch.alignment(scopes.buffer)?;
        let mut scratch_size: DeviceSize = 0;

        let mut skin_cache_offset = 0;
        let mut entity_ids = HashSet::new();
        let mut entity_addresses = Vec::with_capacity(render_snapshot.entities.len());
        let mut builds = Vec::new();

        for entity in render_snapshot.entities.iter() {
            if entity.animation.is_none() {
                entity_addresses.push(mesh_addresses[entity.mesh_id as usize]);

                continue;
            }

            let mesh_vertices = self.mesh_provider
                .with_resource(ResourceId::from(entity.mesh_id), |mesh| mesh.vertices_allocation)
                .context("Animated entity mesh is not resident")?;

            let vertex_address = skin_cache_vertex.range.device_address
                + skin_cache_offset as DeviceSize * MeshVertexGPU::SIZE;

            skin_cache_offset += mesh_vertices.size;

            let Some(geometry_ranges) = blas.geometry_ranges(ResourceId::from(entity.mesh_id)) else {
                entity_addresses.push(mesh_addresses[entity.mesh_id as usize]);

                continue;
            };

            let geometry_ranges = geometry_ranges
                .iter()
                .map(|geometry_range| GeometryRange {
                    vertex_offset: geometry_range.vertex_offset - mesh_vertices.offset,
                    ..*geometry_range
                })
                .collect::<Vec<_>>();

            let primitive_counts = geometry_ranges
                .iter()
                .map(|geometry_range| geometry_range.index_count / 3)
                .collect::<Vec<_>>();

            let plan = blas.plan_skinned(entity.id, &primitive_counts, || {
                let geometries = geometry_ranges
                    .iter()
                    .map(|geometry_range| blas.triangle_geometry(vertex_address, geometry_range))
                    .collect::<Vec<_>>();

                let sizes = frame_context.acceleration_structure_build_sizes(
                    &skinned_blas_build_geometry_info(&geometries),
                    &primitive_counts,
                )?;

                let acceleration_structure = blas.allocate(
                    &format!("blas_entity_{}", entity.id.0),
                    sizes.acceleration_structure_size,
                )?;

                Ok(SkinnedBlasEntry {
                    acceleration_structure,
                    primitive_counts: primitive_counts.clone(),

                    build_scratch_size: sizes.build_scratch_size,
                    update_scratch_size: sizes.update_scratch_size,

                    updates_since_rebuild: 0,
                })
            })?;

            let scratch_offset = align_up(scratch_size, alignment);
            scratch_size = scratch_offset + plan.scratch_size;

            entity_ids.insert(entity.id);
            entity_addresses.push(plan.device_address);

            builds.push(SkinnedBLASBuild {
                geometry_ranges,
                vertex_address,
                plan,
                scratch_offset,
            });
        }

        blas.retain_skinned(&entity_ids);

        self.scratch.reserve_region(scopes.buffer, scratch_size)?;

        self.blas_addresses.stage_slice(scopes.buffer, &entity_addresses)?;

        Ok(SkinnedBLASPassData {
            blas,
            builds,
        })
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        scopes: &RecordScopes,
        data: Self::PassData,
    ) -> Result<()> {
        if data.builds.is_empty() {
            return Ok(());
        }

        let index_stride = size_of::<u32>() as u32;

        let scratch = scopes.buffer.get_physical_buffer(self.scratch);

        let geometries = data.builds
            .iter()
            .map(|build| {
                build.geometry_ranges
                    .iter()
                    .map(|geometry_range| data.blas.triangle_geometry(build.vertex_address, geometry_range))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let range_infos = data.builds
            .iter()
            .map(|build| {
                build.geometry_ranges
                    .iter()
                    .map(|geometry_range| {
                        AccelerationStructureBuildRangeInfoKHR::default()
                            .primitive_count(geometry_range.index_count / 3)
                            .primitive_offset(geometry_range.index_offset * index_stride)
                            .first_vertex(geometry_range.vertex_offset)
                            .transform_offset(0)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let build_geometry_infos = data.builds
            .iter()
            .zip(&geometries)
            .map(|(build, geometry)| {
                let build_geometry_info = skinned_blas_build_geometry_info(geometry)
                    .mode(build.plan.mode)
                    .dst_acceleration_structure(build.plan.handle)
                    .scratch_data(DeviceOrHostAddressKHR {
                        device_address: scratch.range.device_address + build.scratch_offset,
                    });

                if build.plan.mode == BuildAccelerationStructureModeKHR::UPDATE {
                    build_geometry_info.src_acceleration_structure(build.plan.handle)
                } else {
                    build_geometry_info
                }
            })
            .collect::<Vec<_>>();

        let range_slices = range_infos
            .iter()
            .map(|ranges| ranges.as_slice())
            .collect::<Vec<_>>();

        context.build_acceleration_structures(&build_geometry_infos, &range_slices)
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        Ok(())
    }
}
