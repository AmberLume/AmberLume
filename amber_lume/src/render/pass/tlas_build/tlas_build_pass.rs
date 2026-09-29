use anyhow::Result;
use ash::vk::{
    AccelerationStructureBuildRangeInfoKHR, AccessFlags, BuildAccelerationStructureModeKHR,
    DeviceOrHostAddressKHR, PipelineStageFlags,
};
use std::slice;
use gpu::ResourceFactories;
use ray_tracing::TLAS;
use ray_tracing::{instances_geometry, tlas_build_geometry_info};
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
use std::sync::Arc;

pub struct TLASBuildPass {
    tlas_state: VirtualData<Arc<TLAS>>,
    render_snapshot: VirtualData<RenderSnapshot>,

    tlas: VirtualAccelerationStructure,
    blas: VirtualAccelerationStructure,

    instances: VirtualBuffer,
    scratch: VirtualBuffer,
}

impl TLASBuildPass {
    pub fn create(
        tlas_state: VirtualData<Arc<TLAS>>,
        instances: VirtualBuffer,
        scratch: VirtualBuffer,
        blas: VirtualAccelerationStructure,
        tlas: VirtualAccelerationStructure,
        render_snapshot: VirtualData<RenderSnapshot>,
    ) -> Self {
        Self {
            tlas_state,
            render_snapshot,

            tlas,
            blas,

            instances,
            scratch,
        }
    }
}

pub struct TLASBuildPassData {
    entity_count: usize,
    mode: BuildAccelerationStructureModeKHR,
}

impl Pass for TLASBuildPass {
    type PassData = TLASBuildPassData;

    fn name(&self) -> String {
        String::from("tlas_build")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn prepare_data(
        &self,
        scopes: &mut PrepareScopes,
        frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let tlas = scopes.data.get(self.tlas_state).clone();
        let entity_count = scopes.data.get(self.render_snapshot).entities.len();

        if entity_count == 0 {
            return Ok(TLASBuildPassData {
                entity_count,
                mode: BuildAccelerationStructureModeKHR::BUILD,
            });
        }

        let mode = tlas.next_build_mode(entity_count as u32);

        let sizes = frame_context.acceleration_structure_build_sizes(
            &tlas_build_geometry_info(slice::from_ref(&instances_geometry(0))),
            &[entity_count as u32],
        )?;

        let scratch_size = if mode == BuildAccelerationStructureModeKHR::UPDATE {
            sizes.update_scratch_size
        } else {
            sizes.build_scratch_size
        };

        self.scratch.reserve_region(scopes.buffer, scratch_size)?;

        Ok(TLASBuildPassData {
            entity_count,
            mode,
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.tlas_state)
            .consume(self.render_snapshot)
            .read_buffer(
                self.instances,
                AccessFlags::ACCELERATION_STRUCTURE_READ_KHR,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            )
            .read_acceleration_structure(
                self.blas,
                AccessFlags::ACCELERATION_STRUCTURE_READ_KHR,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            )
            .write_acceleration_structure(
                self.tlas,
                AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            )
            .write_buffer(
                self.scratch,
                AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            );
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        scopes: &RecordScopes,
        data: Self::PassData,
    ) -> Result<()> {
        if data.entity_count == 0 {
            return Ok(());
        }

        let instances = scopes.buffer.get_physical_buffer(self.instances);
        let scratch = scopes.buffer.get_physical_buffer(self.scratch);

        let acceleration_structure = scopes
            .acceleration_structure
            .get_physical_acceleration_structure(self.tlas);

        let geometries = [instances_geometry(instances.range.device_address)];
        let mut build_info = tlas_build_geometry_info(&geometries)
            .mode(data.mode)
            .dst_acceleration_structure(acceleration_structure.handle)
            .scratch_data(DeviceOrHostAddressKHR {
                device_address: scratch.range.device_address,
            });
        if data.mode == BuildAccelerationStructureModeKHR::UPDATE {
            build_info = build_info.src_acceleration_structure(acceleration_structure.handle);
        }
        let build_infos = [build_info];

        let ranges = [AccelerationStructureBuildRangeInfoKHR::default()
            .primitive_count(data.entity_count as u32)];
        let range_slices = [ranges.as_slice()];

        context.build_acceleration_structures(&build_infos, &range_slices)
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        Ok(())
    }
}
