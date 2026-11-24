use anyhow::Result;
use ash::vk::{
    AccelerationStructureBuildRangeInfoKHR, AccessFlags, DeviceOrHostAddressKHR, PipelineStageFlags,
};
use std::slice;
use gpu::ResourceFactories;
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

pub struct TLASBuildPass {
    render_snapshot: VirtualData<RenderSnapshot>,

    tlas: VirtualAccelerationStructure,
    blas: VirtualAccelerationStructure,

    instances: VirtualBuffer,
    scratch: VirtualBuffer,
}

impl TLASBuildPass {
    pub fn create(
        instances: VirtualBuffer,
        scratch: VirtualBuffer,
        blas: VirtualAccelerationStructure,
        tlas: VirtualAccelerationStructure,
        render_snapshot: VirtualData<RenderSnapshot>,
    ) -> Self {
        Self {
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
        let entity_count = scopes.data.get(self.render_snapshot).entities.len();

        if entity_count == 0 {
            return Ok(TLASBuildPassData {
                entity_count,
            });
        }

        let sizes = frame_context.acceleration_structure_build_sizes(
            &tlas_build_geometry_info(slice::from_ref(&instances_geometry(0))),
            &[entity_count as u32],
        )?;

        self.scratch.reserve_region(scopes.buffer, sizes.build_scratch_size)?;

        Ok(TLASBuildPassData {
            entity_count,
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
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
        let build_info = tlas_build_geometry_info(&geometries)
            .dst_acceleration_structure(acceleration_structure.handle)
            .scratch_data(DeviceOrHostAddressKHR {
                device_address: scratch.range.device_address,
            });
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
