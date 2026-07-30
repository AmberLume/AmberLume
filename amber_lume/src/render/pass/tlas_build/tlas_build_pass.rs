use gpu::ResourceFactories;
use crate::render::pass::frame_data_context::FrameDataContext;
use crate::render::pass::pass_context::PassContext;
use crate::render::ray_tracing::ray_tracing::RayTracing;
use crate::render::ray_tracing::tlas::{instances_geometry, tlas_build_geometry_info};
use crate::render::render_graph::pass::Pass;
use crate::render::render_graph::pass_resource_declaration::pass_resource_declaration::PassResourceDeclaration;
use crate::render::render_graph::virtual_acceleration_structure::virtual_acceleration_structure::VirtualAccelerationStructure;
use crate::render::render_graph::virtual_buffer::heap_allocator::HeapAllocator;
use crate::render::render_graph::virtual_buffer::virtual_buffer::VirtualBuffer;
use crate::render::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::render::resource_scope::image_resource_scope::ImageResourceScope;
use anyhow::Result;
use ash::vk::{
    AccelerationStructureBuildRangeInfoKHR, AccessFlags, BuildAccelerationStructureModeKHR,
    DeviceOrHostAddressKHR, PipelineStageFlags,
};
use std::sync::Arc;

pub struct TLASBuildPass {
    ray_tracing: Arc<RayTracing>,
    instances: VirtualBuffer,
    blas: VirtualAccelerationStructure,
    tlas: VirtualAccelerationStructure,
}

impl TLASBuildPass {
    pub fn create(
        ray_tracing: Arc<RayTracing>,
        instances: VirtualBuffer,
        blas: VirtualAccelerationStructure,
        tlas: VirtualAccelerationStructure,
    ) -> Self {
        Self {
            ray_tracing,
            instances,
            blas,
            tlas,
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

    fn is_enabled(&self, _context: &FrameDataContext) -> bool {
        true
    }

    fn prepare_data(
        &self,
        context: &FrameDataContext,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        Ok(TLASBuildPassData {
            entity_count: context.render_snapshot.entities.len(),
        })
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
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
        context: &PassContext,
        _image_scope: &ImageResourceScope,
        buffer_scope: &BufferResourceScope,
        data: Self::PassData,
    ) -> Result<()> {
        if data.entity_count == 0 {
            return Ok(());
        }

        let instances = buffer_scope.get_physical_buffer(self.instances);
        let command_buffer = context.command_recording.command_buffer;

        let slot = context.frame_index.value as usize;
        let tlas = &self.ray_tracing.tlas[slot];
        let mode = tlas.next_build_mode(data.entity_count as u32);

        let geometries = [instances_geometry(instances.device_address)];
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
            self.ray_tracing.as_loader.cmd_build_acceleration_structures(
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
