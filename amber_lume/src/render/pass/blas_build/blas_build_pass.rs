use anyhow::{Context, Result};
use ash::vk::{
    AccelerationStructureBuildRangeInfoKHR, AccelerationStructureKHR,
    AccessFlags, BuildAccelerationStructureFlagsKHR, BuildAccelerationStructureModeKHR,
    DeviceAddress, DeviceOrHostAddressKHR, DeviceSize, PipelineStageFlags,
};
use gpu::GpuSize;
use gpu::ResourceFactories;
use gpu_data::VertexPositionGPU;
use index_allocator::ResourceId;
use resource_store::FrameSliceIndex;
use resource_store::GeometryRange;
use ray_tracing::blas_build_geometry_info;
use ray_tracing::triangle_geometry;
use ray_tracing::align_up;
use ray_tracing::BLAS;
use render_graph::PrepareScopes;
use render_graph::RecordScopes;
use render_graph::DataResourceScope;
use render_graph::FrameContext;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::VirtualAccelerationStructure;
use render_graph::VirtualBuffer;
use render_graph::VirtualData;
use render_snapshot::RenderSnapshot;
use resource_residency::ResourceProvider;
use resource_store::SkinBackend;
use std::mem::size_of;
use std::sync::Arc;

struct BLASBuild {
    geometry_ranges: Vec<GeometryRange>,
    vertex_address: DeviceAddress,
    handle: AccelerationStructureKHR,
    flags: BuildAccelerationStructureFlagsKHR,
    mode: BuildAccelerationStructureModeKHR,
    scratch_offset: DeviceSize,
}

pub struct BLASBuildPassData {
    blas_builds: Vec<BLASBuild>,
}

pub struct BLASBuildPass {
    blas_state: VirtualData<Arc<BLAS>>,
    render_snapshot: VirtualData<RenderSnapshot>,
    skin_slice_index: VirtualData<FrameSliceIndex>,

    blas: VirtualAccelerationStructure,

    blas_addresses: VirtualBuffer,
    index_buffer: VirtualBuffer,
    vertex_position_buffer: VirtualBuffer,
    scratch: VirtualBuffer,

    skin_provider: Arc<ResourceProvider<SkinBackend>>,
}

impl BLASBuildPass {
    pub fn create(
        blas_state: VirtualData<Arc<BLAS>>,
        render_snapshot: VirtualData<RenderSnapshot>,
        skin_slice_index: VirtualData<FrameSliceIndex>,
        blas: VirtualAccelerationStructure,
        blas_addresses: VirtualBuffer,
        scratch: VirtualBuffer,
        vertex_position_buffer: VirtualBuffer,
        index_buffer: VirtualBuffer,
        skin_provider: Arc<ResourceProvider<SkinBackend>>,
    ) -> Self {
        Self {
            blas_state,
            render_snapshot,
            skin_slice_index,

            blas,

            blas_addresses,
            index_buffer,
            vertex_position_buffer,
            scratch,

            skin_provider,
        }
    }
}

impl Pass for BLASBuildPass {
    type PassData = BLASBuildPassData;

    fn name(&self) -> String {
        String::from("blas_build")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.blas_state)
            .consume(self.render_snapshot)
            .consume(self.skin_slice_index)
            .write_acceleration_structure(
                self.blas,
                AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            )
            .read_buffer(
                self.vertex_position_buffer,
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
        let skin_slice_index = *scopes.data.get(self.skin_slice_index);

        let vertex_position_buffer = scopes.buffer.get_physical_buffer(self.vertex_position_buffer);
        let index_buffer = scopes.buffer.get_physical_buffer(self.index_buffer);

        let alignment = self.scratch.alignment(scopes.buffer)?;
        let mut scratch_size: DeviceSize = 0;

        let mut blas_builds = Vec::new();

        let pending = blas.consume_events(|geometry_ranges, flags| {
            let geometries = geometry_ranges
                .iter()
                .map(|geometry_range| triangle_geometry(vertex_position_buffer.range.device_address, index_buffer.range.device_address, geometry_range))
                .collect::<Vec<_>>();
            let primitive_counts = geometry_ranges
                .iter()
                .map(|geometry_range| geometry_range.index_count / 3)
                .collect::<Vec<_>>();

            frame_context.acceleration_structure_build_sizes(
                &blas_build_geometry_info(&geometries, flags),
                &primitive_counts,
            )
        })?;

        for mesh_id in pending {
            blas.with_entry(mesh_id, |entry| {
                let scratch_offset = align_up(scratch_size, alignment);
                scratch_size = scratch_offset + entry.build_scratch_size;

                blas_builds.push(BLASBuild {
                    geometry_ranges: entry.geometry_ranges.clone(),
                    vertex_address: vertex_position_buffer.range.device_address,
                    handle: entry.acceleration_structure.handle,
                    flags: BLAS::STATIC_FLAGS,
                    mode: BuildAccelerationStructureModeKHR::BUILD,
                    scratch_offset,
                });
            });
        }

        let mut entity_addresses = Vec::with_capacity(render_snapshot.entities.len());

        for entity in render_snapshot.entities.iter() {
            let Some(animation) = entity.animation.as_ref() else {
                entity_addresses.push(
                    blas.with_entry(ResourceId::from(entity.mesh_id), |entry| entry.acceleration_structure.device_address)
                        .unwrap_or(0),
                );

                continue;
            };

            let mesh_id = self.skin_provider
                .with_resource(ResourceId::from(animation.skin_id), |skin| skin.mesh_id)
                .context("Skin is not resident")?;

            blas
                .with_entry(mesh_id, |entry| {
                    let mode = entry.refit_mode();

                    let scratch_offset = align_up(scratch_size, alignment);
                    scratch_size = scratch_offset + if mode == BuildAccelerationStructureModeKHR::UPDATE {
                        entry.update_scratch_size
                    } else {
                        entry.build_scratch_size
                    };

                    entity_addresses.push(entry.acceleration_structure.device_address);

                    blas_builds.push(BLASBuild {
                        geometry_ranges: entry.geometry_ranges.clone(),
                        vertex_address: vertex_position_buffer.range.device_address
                            + (skin_slice_index.current * entry.vertex_slice_stride) as DeviceSize * VertexPositionGPU::SIZE,
                        handle: entry.acceleration_structure.handle,
                        flags: BLAS::SKINNED_FLAGS,
                        mode,
                        scratch_offset,
                    });
                })
                .with_context(|| format!("Mesh {} has no BLAS entry", mesh_id.inner))?;
        }

        self.scratch.reserve_region(scopes.buffer, scratch_size)?;

        self.blas_addresses.stage_slice(scopes.buffer, &entity_addresses)?;

        Ok(BLASBuildPassData {
            blas_builds,
        })
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        scopes: &RecordScopes,
        data: Self::PassData,
    ) -> Result<()> {
        if data.blas_builds.is_empty() {
            return Ok(());
        }

        let index_stride = size_of::<u32>() as u32;

        let scratch = scopes.buffer.get_physical_buffer(self.scratch);
        let index_buffer = scopes.buffer.get_physical_buffer(self.index_buffer);

        let geometries = data.blas_builds
            .iter()
            .map(|blas_build| {
                blas_build.geometry_ranges
                    .iter()
                    .map(|geometry_range| triangle_geometry(blas_build.vertex_address, index_buffer.range.device_address, geometry_range))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let range_infos = data.blas_builds
            .iter()
            .map(|blas_build| {
                blas_build.geometry_ranges
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

        let build_geometry_infos = data.blas_builds
            .iter()
            .zip(&geometries)
            .map(|(blas_build, geometry)| {
                let build_geometry_info = blas_build_geometry_info(geometry, blas_build.flags)
                    .mode(blas_build.mode)
                    .dst_acceleration_structure(blas_build.handle)
                    .scratch_data(DeviceOrHostAddressKHR {
                        device_address: scratch.range.device_address + blas_build.scratch_offset,
                    });

                if blas_build.mode == BuildAccelerationStructureModeKHR::UPDATE {
                    build_geometry_info.src_acceleration_structure(blas_build.handle)
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
