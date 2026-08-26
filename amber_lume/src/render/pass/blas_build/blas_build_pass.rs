use anyhow::Result;
use ash::vk::{
    AccelerationStructureBuildRangeInfoKHR, AccelerationStructureKHR,
    AccessFlags, DeviceOrHostAddressKHR, DeviceSize, PipelineStageFlags,
};
use gpu::ResourceFactories;
use render_snapshot::RenderSnapshot;
use resource_store::GeometryRange;
use ray_tracing::blas_build_geometry_info;
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
use std::mem::size_of;
use std::sync::Arc;

struct BLASBuild {
    geometry_ranges: Vec<GeometryRange>,
    handle: AccelerationStructureKHR,
    scratch_offset: DeviceSize,
}

pub struct BLASBuildPassData {
    blas: Arc<BLAS>,
    blas_builds: Vec<BLASBuild>,
}

pub struct BLASBuildPass {
    blas_state: VirtualData<Arc<BLAS>>,
    render_snapshot: VirtualData<RenderSnapshot>,

    blas: VirtualAccelerationStructure,
    addresses: VirtualBuffer,
    scratch: VirtualBuffer,
    mesh_vertex_buffer: VirtualBuffer,
    index_buffer: VirtualBuffer,
}

impl BLASBuildPass {
    pub fn create(
        blas_state: VirtualData<Arc<BLAS>>,
        render_snapshot: VirtualData<RenderSnapshot>,
        blas: VirtualAccelerationStructure,
        addresses: VirtualBuffer,
        scratch: VirtualBuffer,
        mesh_vertex_buffer: VirtualBuffer,
        index_buffer: VirtualBuffer,
    ) -> Self {
        Self {
            blas_state,
            render_snapshot,

            blas,
            addresses,
            scratch,
            mesh_vertex_buffer,
            index_buffer,
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
            .write_acceleration_structure(
                self.blas,
                AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            )
            .read_buffer(
                self.mesh_vertex_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            )
            .read_buffer(
                self.index_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            )
            .write_buffer(
                self.addresses,
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
        let geometry_changes = &scopes.data.get(self.render_snapshot).geometry_changes;

        for mesh_id in &geometry_changes.unloaded {
            blas.unregister(*mesh_id);
        }

        for loaded in &geometry_changes.loaded {
            blas.record_geometry(loaded.mesh_id, loaded.ranges.clone());
        }

        let pending = geometry_changes
            .loaded
            .iter()
            .map(|loaded| loaded.mesh_id)
            .chain(geometry_changes.changed.iter().copied())
            .collect::<Vec<_>>();

        let alignment = self.scratch.alignment(scopes.buffer)?;
        let mut scratch_size: DeviceSize = 0;

        let mut blas_builds = Vec::with_capacity(pending.len());

        for mesh_id in pending {
            let Some(geometry_ranges) = blas.geometry_ranges(mesh_id) else {
                continue;
            };

            let geometries = geometry_ranges
                .iter()
                .map(|geometry_range| blas.triangle_geometry(geometry_range))
                .collect::<Vec<_>>();
            let primitive_counts = geometry_ranges
                .iter()
                .map(|geometry_range| geometry_range.index_count / 3)
                .collect::<Vec<_>>();

            let size_geometry_info = blas_build_geometry_info(&geometries);

            let sizes = frame_context
                .acceleration_structure_build_sizes(&size_geometry_info, &primitive_counts)?;

            let scratch_offset = align_up(scratch_size, alignment);
            scratch_size = scratch_offset + sizes.build_scratch_size;

            let acceleration_structure = blas.allocate(
                &format!("blas_mesh_{}", mesh_id.inner),
                sizes.acceleration_structure_size,
            )?;

            let handle = blas.register(mesh_id, acceleration_structure);

            blas_builds.push(BLASBuild {
                geometry_ranges,
                handle,
                scratch_offset,
            });
        }

        self.scratch.reserve_region(scopes.buffer, scratch_size)?;

        self.addresses.stage_slice(scopes.buffer, &blas.addresses())?;

        Ok(BLASBuildPassData {
            blas,
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
        
        let scratch_address = scopes.buffer.get_physical_buffer(self.scratch);

        let mut geometries = Vec::new();
        let mut range_infos = Vec::new();

        for blas_build in &data.blas_builds {
            let geometry = blas_build
                .geometry_ranges
                .iter()
                .map(|geometry_range| data.blas.triangle_geometry(geometry_range))
                .collect::<Vec<_>>();

            let mut build_range_infos = Vec::new();

            for geometry_range in &blas_build.geometry_ranges {
                let range_info = AccelerationStructureBuildRangeInfoKHR::default()
                    .primitive_count(geometry_range.index_count / 3)
                    .primitive_offset(geometry_range.index_offset * index_stride)
                    .first_vertex(geometry_range.vertex_offset)
                    .transform_offset(0);

                build_range_infos.push(range_info);
            }

            geometries.push(geometry);
            range_infos.push(build_range_infos);
        }

        let mut build_geometry_infos = Vec::new();
        let mut range_slices = Vec::new();

        for ((blas_build, geometry), build_range_infos) in data
            .blas_builds
            .iter()
            .zip(&geometries)
            .zip(&range_infos)
        {
            let build_geometry_info = blas_build_geometry_info(geometry)
                .dst_acceleration_structure(blas_build.handle)
                .scratch_data(DeviceOrHostAddressKHR {
                    device_address: scratch_address.range.device_address + blas_build.scratch_offset,
                });

            build_geometry_infos.push(build_geometry_info);
            range_slices.push(build_range_infos.as_slice());
        }

        context.build_acceleration_structures(&build_geometry_infos, &range_slices)
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        Ok(())
    }
}
