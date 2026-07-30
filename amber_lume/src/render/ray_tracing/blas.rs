use gpu::FrameIndex;
use crate::limits::RenderLimits;
use gpu::ManagedBuffer;
use gpu::ManagedBufferFactory;
use gpu::BufferView;
use gpu::ResourceFactories;
use crate::render::ray_tracing::acceleration_structure_factory::AccelerationStructureFactory;
use crate::render::ray_tracing::blas_registry::BLASRegistry;
use crate::render::ray_tracing::blas_request_queue::BLASRequestQueue;
use crate::render::ray_tracing::managed_acceleration_structure::ManagedAccelerationStructure;
use crate::render::ray_tracing::ray_tracing::align_up;
use crate::render::ray_tracing::rt_limits::RTLimits;
use gpu::ResourceTransfer;
use gpu::DeferredDestroy;
use crate::resources::resource_buffers::ResourceBuffers;
use crate::resources::store::providers::mesh::buffer::vertex_buffer::VertexGPU;
use crate::resources::store::providers::resource_provider::ResourceId;
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
use gpu_allocator::MemoryLocation;
use std::mem::size_of;
use std::slice;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

pub struct BLAS {
    pub registry: BLASRegistry,
    pub request_queue: Arc<BLASRequestQueue>,
    pub geometry: AccelerationStructureGeometryKHR<'static>,
    pub scratch_capacity: DeviceSize,

    pub addresses_buffer: ManagedBuffer,

    scratch_buffers: Vec<ManagedBuffer>,
    pub destroy_queue: DeferredDestroy<ManagedAccelerationStructure>,

    resource_transfer: Arc<ResourceTransfer>,

    rt_limits: RTLimits,
}

impl BLAS {
    pub fn new(
        limits: &RenderLimits,
        rt_limits: RTLimits,
        as_loader: &AccelerationStructureDevice,
        factory: Arc<AccelerationStructureFactory>,
        resource_factories: Arc<ResourceFactories>,
        resource_transfer: Arc<ResourceTransfer>,
        request_queue: Arc<BLASRequestQueue>,
        frame_counter: Arc<AtomicU64>,
        resource_buffers: &ResourceBuffers,
    ) -> Result<Self> {
        let buffer_factory = &resource_factories.buffer_factory;

        let geometry = triangle_geometry(resource_buffers, limits);

        let scratch_capacity = worst_case_scratch_size(as_loader, &geometry, limits);
        let scratch_size = scratch_capacity + rt_limits.min_scratch_offset_alignment as DeviceSize;

        let scratch_buffers = (0..limits.frames_in_flight)
            .map(|index| {
                buffer_factory.create_managed_buffer(
                    &format!("blas_scratch_{index}"),
                    scratch_size,
                    BufferUsageFlags::STORAGE_BUFFER,
                    MemoryLocation::GpuOnly,
                )
            })
            .collect::<Result<Vec<_>>>()?;

        let address_stride = size_of::<DeviceAddress>() as DeviceSize;
        let addresses_size = address_stride * limits.resource_limits.max_meshes as DeviceSize;
        let addresses_buffer = buffer_factory.create_managed_buffer(
            "blas_addresses",
            addresses_size,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
        )?;
        resource_transfer.load_buffer_at(
            &BufferView::create(&addresses_buffer, 0, addresses_size),
            &vec![0 as DeviceAddress; limits.resource_limits.max_meshes as usize],
        )?;

        let destroy_queue = {
            let resource_factories = resource_factories.clone();

            DeferredDestroy::new(
                limits.frames_in_flight,
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

            scratch_buffers,
            addresses_buffer,
            destroy_queue,

            resource_transfer,
            rt_limits,
        })
    }

    pub fn has_pending(&self) -> bool {
        !self.request_queue.is_empty()
    }

    pub fn scratch_address(&self, frame_index: FrameIndex) -> DeviceAddress {
        align_up(
            self.scratch_buffers[frame_index.value as usize].device_address,
            self.rt_limits.min_scratch_offset_alignment as DeviceSize,
        )
    }

    pub fn write_address(&self, mesh_id: ResourceId, address: DeviceAddress) -> Result<()> {
        let stride = size_of::<DeviceAddress>() as DeviceSize;

        self.resource_transfer.load_buffer_at(
            &BufferView::create(&self.addresses_buffer, mesh_id as DeviceSize * stride, stride),
            &[address],
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

        buffer_factory.destroy_buffer(self.addresses_buffer)?;

        for scratch in self.scratch_buffers {
            buffer_factory.destroy_buffer(scratch)?;
        }

        Ok(())
    }
}

fn triangle_geometry(
    resource_buffers: &ResourceBuffers,
    limits: &RenderLimits,
) -> AccelerationStructureGeometryKHR<'static> {
    let triangles = AccelerationStructureGeometryTrianglesDataKHR::default()
        .vertex_format(Format::R32G32B32_SFLOAT)
        .vertex_data(DeviceOrHostAddressConstKHR {
            device_address: resource_buffers.vertex_buffer,
        })
        .vertex_stride(size_of::<VertexGPU>() as DeviceSize)
        .max_vertex(limits.resource_limits.max_vertices.saturating_sub(1))
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
    limits: &RenderLimits,
) -> DeviceSize {
    let mut sizes = AccelerationStructureBuildSizesInfoKHR::default();

    unsafe {
        as_loader.get_acceleration_structure_build_sizes(
            AccelerationStructureBuildTypeKHR::DEVICE,
            &blas_build_geometry_info(slice::from_ref(geometry)),
            &[limits.resource_limits.max_indices / 3],
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
