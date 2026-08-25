use crate::acceleration_structure_factory::AccelerationStructureFactory;
use crate::blas_registry::BLASRegistry;
use crate::blas_request_queue::BLASRequestQueue;
use crate::managed_acceleration_structure::ManagedAccelerationStructure;
use anyhow::Result;
use ash::vk::{
    AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureGeometryDataKHR,
    AccelerationStructureGeometryKHR, AccelerationStructureGeometryTrianglesDataKHR,
    AccelerationStructureTypeKHR, BuildAccelerationStructureFlagsKHR,
    BuildAccelerationStructureModeKHR, DeviceAddress, DeviceOrHostAddressConstKHR, DeviceSize,
    Format, GeometryFlagsKHR, GeometryTypeKHR, IndexType,
};
use gpu::ManagedBufferFactory;
use gpu::ResourceFactories;
use gpu_data::MeshVertexGPU;
use index_allocator::DeferredDestroy;
use index_allocator::ResourceLimits;
use resource_store::ResourceBuffers;
use std::mem::size_of;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

pub struct BLAS {
    pub registry: BLASRegistry,
    pub request_queue: Arc<BLASRequestQueue>,
    pub geometry: AccelerationStructureGeometryKHR<'static>,

    pub destroy_queue: DeferredDestroy<ManagedAccelerationStructure>,

    max_meshes: u32,
}

impl BLAS {
    pub(crate) fn new(
        frames_in_flight: u32,
        resource_limits: ResourceLimits,
        factory: Arc<AccelerationStructureFactory>,
        resource_factories: Arc<ResourceFactories>,
        request_queue: Arc<BLASRequestQueue>,
        frame_counter: Arc<AtomicU64>,
        resource_buffers: &ResourceBuffers,
    ) -> Result<Self> {
        let geometry = triangle_geometry(resource_buffers, resource_limits);

        let destroy_queue = {
            let resource_factories = resource_factories.clone();

            DeferredDestroy::new(
                frames_in_flight,
                frame_counter,
                move |acceleration_structure| {
                    factory.destroy(&resource_factories.buffer_factory, acceleration_structure)
                },
            )
        };

        Ok(Self {
            registry: BLASRegistry::new(),
            request_queue,
            geometry,

            destroy_queue,

            max_meshes: resource_limits.max_meshes,
        })
    }

    pub fn addresses(&self) -> Vec<DeviceAddress> {
        self.registry.addresses(self.max_meshes as usize)
    }

    pub fn destroy(
        self,
        factory: &AccelerationStructureFactory,
        buffer_factory: &ManagedBufferFactory,
    ) -> Result<()> {
        for acceleration_structure in self.registry.drain() {
            factory.destroy(buffer_factory, acceleration_structure)?;
        }

        self.destroy_queue.destroy_all()?;

        Ok(())
    }
}

fn triangle_geometry(
    resource_buffers: &ResourceBuffers,
    resource_limits: ResourceLimits,
) -> AccelerationStructureGeometryKHR<'static> {
    let triangles = AccelerationStructureGeometryTrianglesDataKHR::default()
        .vertex_format(Format::R32G32B32_SFLOAT)
        .vertex_data(DeviceOrHostAddressConstKHR {
            device_address: resource_buffers.mesh_vertex_buffer.device_address,
        })
        .vertex_stride(size_of::<MeshVertexGPU>() as DeviceSize)
        .max_vertex(resource_limits.max_vertices.saturating_sub(1))
        .index_type(IndexType::UINT32)
        .index_data(DeviceOrHostAddressConstKHR {
            device_address: resource_buffers.index_buffer.device_address,
        });

    AccelerationStructureGeometryKHR::default()
        .geometry_type(GeometryTypeKHR::TRIANGLES)
        .geometry(AccelerationStructureGeometryDataKHR { triangles })
        .flags(GeometryFlagsKHR::OPAQUE)
}

pub fn blas_build_geometry_info<'a>(
    geometries: &'a [AccelerationStructureGeometryKHR<'a>],
) -> AccelerationStructureBuildGeometryInfoKHR<'a> {
    AccelerationStructureBuildGeometryInfoKHR::default()
        .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL)
        .flags(BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
        .mode(BuildAccelerationStructureModeKHR::BUILD)
        .geometries(geometries)
}
