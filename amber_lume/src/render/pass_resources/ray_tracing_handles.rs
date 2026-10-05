use ash::vk::DeviceSize;
use gpu::RayTracingProperties;
use render_graph::PassGraph;
use render_graph::VirtualAccelerationStructure;
use render_graph::VirtualBuffer;

#[derive(Clone, Copy)]
pub struct RayTracingHandles {
    pub blas: VirtualAccelerationStructure,
    pub tlas: VirtualAccelerationStructure,

    pub blas_addresses: VirtualBuffer,
    pub blas_scratch: VirtualBuffer,
    pub tlas_instances: VirtualBuffer,
    pub tlas_scratch: VirtualBuffer,
}

impl RayTracingHandles {
    pub fn create(pass_graph: &mut PassGraph, properties: &RayTracingProperties) -> Self {
        Self {
            blas: pass_graph.import_acceleration_structure("blas"),
            tlas: pass_graph.import_acceleration_structure("tlas"),

            blas_addresses: pass_graph.create_upload_buffer("blas_addresses", false),
            blas_scratch: pass_graph.create_scratch_buffer("blas_scratch", properties.min_scratch_offset_alignment as DeviceSize),
            tlas_scratch: pass_graph.create_scratch_buffer("tlas_scratch", properties.min_scratch_offset_alignment as DeviceSize),
            tlas_instances: pass_graph.create_device_buffer("tlas_instances", false),
        }
    }
}
