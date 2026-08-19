use crate::acceleration_structure_factory::AccelerationStructureFactory;
use crate::blas_registry::BLASRegistry;
use crate::blas_request_queue::BLASRequestQueue;
use crate::managed_acceleration_structure::ManagedAccelerationStructure;
use crate::ray_tracing::align_up;
use crate::rt_limits::RTLimits;
use anyhow::Result;
use ash::khr::acceleration_structure::Device as AccelerationStructureDevice;
use ash::vk::{
    AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureBuildSizesInfoKHR,
    AccelerationStructureBuildTypeKHR, AccelerationStructureGeometryDataKHR,
    AccelerationStructureGeometryKHR, AccelerationStructureGeometryTrianglesDataKHR,
    AccelerationStructureTypeKHR, BufferUsageFlags, BuildAccelerationStructureFlagsKHR,
    BuildAccelerationStructureModeKHR, DeviceAddress, DeviceOrHostAddressConstKHR, DeviceSize,
    Format, GeometryFlagsKHR, GeometryTypeKHR, IndexType,
};
use gpu::ManagedBuffer;
use gpu::ManagedBufferFactory;
use gpu::ResourceFactories;
use gpu_allocator::MemoryLocation;
use gpu_data::VertexGPU;
use index_allocator::DeferredDestroy;
use index_allocator::FrameIndex;
use index_allocator::ResourceLimits;
use resource_store::ResourceBuffers;
use std::mem::size_of;
use std::slice;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

pub struct BLAS {
    pub registry: BLASRegistry,
    pub request_queue: Arc<BLASRequestQueue>,
    pub geometry: AccelerationStructureGeometryKHR<'static>,
    pub scratch_capacity: DeviceSize,

    pub destroy_queue: DeferredDestroy<ManagedAccelerationStructure>,

    scratch_buffers: Vec<ManagedBuffer>,

    max_meshes: u32,
    rt_limits: RTLimits,
}

impl BLAS {
    pub(crate) fn new(
        frames_in_flight: u32,
        resource_limits: ResourceLimits,
        rt_limits: RTLimits,
        as_loader: &AccelerationStructureDevice,
        factory: Arc<AccelerationStructureFactory>,
        resource_factories: Arc<ResourceFactories>,
        request_queue: Arc<BLASRequestQueue>,
        frame_counter: Arc<AtomicU64>,
        resource_buffers: &ResourceBuffers,
    ) -> Result<Self> {
        let buffer_factory = &resource_factories.buffer_factory;

        let geometry = triangle_geometry(resource_buffers, resource_limits);

        let scratch_capacity = worst_case_scratch_size(as_loader, &geometry, resource_limits);
        let scratch_size = scratch_capacity + rt_limits.min_scratch_offset_alignment as DeviceSize;

        let scratch_buffers = (0..frames_in_flight)
            .map(|index| {
                buffer_factory.create_managed_buffer(
                    &format!("blas_scratch_{index}"),
                    scratch_size,
                    BufferUsageFlags::STORAGE_BUFFER,
                    MemoryLocation::GpuOnly,
                )
            })
            .collect::<Result<Vec<_>>>()?;

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
            scratch_capacity,

            destroy_queue,

            scratch_buffers,

            max_meshes: resource_limits.max_meshes,
            rt_limits,
        })
    }

    pub fn addresses(&self) -> Vec<DeviceAddress> {
        self.registry.addresses(self.max_meshes as usize)
    }

    pub fn scratch_address(&self, frame_index: FrameIndex) -> DeviceAddress {
        align_up(
            self.scratch_buffers[frame_index.value as usize].device_address,
            self.rt_limits.min_scratch_offset_alignment as DeviceSize,
        )
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

        for scratch in self.scratch_buffers {
            buffer_factory.destroy_buffer(scratch)?;
        }

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
            device_address: resource_buffers.vertex_buffer,
        })
        .vertex_stride(size_of::<VertexGPU>() as DeviceSize)
        .max_vertex(resource_limits.max_vertices.saturating_sub(1))
        .index_type(IndexType::UINT32)
        .index_data(DeviceOrHostAddressConstKHR {
            device_address: resource_buffers.index_buffer,
        });

    AccelerationStructureGeometryKHR::default()
        .geometry_type(GeometryTypeKHR::TRIANGLES)
        .geometry(AccelerationStructureGeometryDataKHR { triangles })
        .flags(GeometryFlagsKHR::OPAQUE)
}

fn worst_case_scratch_size(
    as_loader: &AccelerationStructureDevice,
    geometry: &AccelerationStructureGeometryKHR<'static>,
    resource_limits: ResourceLimits,
) -> DeviceSize {
    let mut sizes = AccelerationStructureBuildSizesInfoKHR::default();

    unsafe {
        as_loader.get_acceleration_structure_build_sizes(
            AccelerationStructureBuildTypeKHR::DEVICE,
            &blas_build_geometry_info(slice::from_ref(geometry)),
            &[resource_limits.max_indices / 3],
            &mut sizes,
        );
    }

    sizes.build_scratch_size
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
