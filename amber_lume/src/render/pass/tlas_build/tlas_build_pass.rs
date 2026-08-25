use render_graph::ReadbackScope;
use render_graph::VirtualData;
use render_snapshot::RenderSnapshot;
use gpu::ResourceFactories;
use render_graph::FrameContext;
use ray_tracing::RayTracing;
use ray_tracing::{instances_geometry, tlas_build_geometry_info};
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::VirtualAccelerationStructure;
use render_graph::VirtualBuffer;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::ImageResourceScope;
use anyhow::Result;
use ash::vk::{
    AccelerationStructureBuildRangeInfoKHR, AccessFlags, BuildAccelerationStructureModeKHR,
    DeviceOrHostAddressKHR, PipelineStageFlags,
};
use std::sync::Arc;

pub struct TLASBuildPass {
    ray_tracing: VirtualData<Arc<RayTracing>>,
    instances: VirtualBuffer,
    blas: VirtualAccelerationStructure,
    tlas: VirtualAccelerationStructure,

    render_snapshot: VirtualData<RenderSnapshot>,
}

impl TLASBuildPass {
    pub fn create(
        ray_tracing: VirtualData<Arc<RayTracing>>,
        instances: VirtualBuffer,
        blas: VirtualAccelerationStructure,
        tlas: VirtualAccelerationStructure,
        render_snapshot: VirtualData<RenderSnapshot>,
    ) -> Self {
        Self {
            ray_tracing,
            instances,
            blas,
            tlas,

            render_snapshot,
        }
    }
}

pub struct TLASBuildPassData {
    ray_tracing: Arc<RayTracing>,
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
        data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let ray_tracing = data_scope.get(self.ray_tracing).clone();
        let render_snapshot = data_scope.get(self.render_snapshot);

        Ok(TLASBuildPassData {
            ray_tracing,
            entity_count: render_snapshot.entities.len(),
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.ray_tracing)
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
            );
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        _image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        _readback_scope: &ReadbackScope,
        data: Self::PassData,
    ) -> Result<()> {
        if data.entity_count == 0 {
            return Ok(());
        }

        let instances = buffer_scope.get_physical_buffer(self.instances);
        let command_buffer = context.command_buffer();

        let slot = context.frame_index.value as usize;
        let tlas = &data.ray_tracing.tlas[slot];
        let mode = tlas.next_build_mode(data.entity_count as u32);

        let geometries = [instances_geometry(instances.range.device_address)];
        let mut build_info = tlas_build_geometry_info(&geometries)
            .mode(mode)
            .dst_acceleration_structure(tlas.acceleration_structure.handle)
            .scratch_data(DeviceOrHostAddressKHR {
                device_address: tlas.scratch_address(),
            });
        if mode == BuildAccelerationStructureModeKHR::UPDATE {
            build_info = build_info.src_acceleration_structure(tlas.acceleration_structure.handle);
        }
        let build_infos = [build_info];

        let ranges = [AccelerationStructureBuildRangeInfoKHR::default()
            .primitive_count(data.entity_count as u32)];
        let range_slices = [ranges.as_slice()];

        unsafe {
            data.ray_tracing.context.device.cmd_build_acceleration_structures(
                command_buffer,
                &build_infos,
                &range_slices,
            );
        }

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        Ok(())
    }
}
