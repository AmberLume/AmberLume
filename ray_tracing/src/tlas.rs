use gpu::ResourceFactories;
use gpu::ManagedAccelerationStructure;
use gpu::ManagedAccelerationStructureDescriptorSet;
use crate::ray_tracing::align_up;
use gpu::RayTracingContext;
use anyhow::bail;
use anyhow::Result;
use ash::vk::{
    AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureBuildSizesInfoKHR,
    AccelerationStructureBuildTypeKHR, AccelerationStructureGeometryDataKHR,
    AccelerationStructureGeometryInstancesDataKHR, AccelerationStructureGeometryKHR,
    AccelerationStructureTypeKHR, BufferUsageFlags, BuildAccelerationStructureFlagsKHR,
    BuildAccelerationStructureModeKHR, DeviceAddress, DeviceOrHostAddressConstKHR, DeviceSize,
    GeometryTypeKHR,
};
use gpu::ManagedBuffer;
use gpu_allocator::MemoryLocation;
use index_allocator::ResourceLimits;
use std::slice;
use std::sync::atomic::{AtomicU32, Ordering};

const REBUILD_INTERVAL: u32 = 60;

pub struct TLAS {
    pub acceleration_structure: ManagedAccelerationStructure,

    scratch: ManagedBuffer,
    alignment: DeviceSize,

    last_built_count: AtomicU32,
    updates_since_rebuild: AtomicU32,
}

impl TLAS {
    pub(crate) fn new(
        frame_index: u32,
        resource_limits: ResourceLimits,
        context: &RayTracingContext,
        resource_factories: &ResourceFactories,
        acceleration_structures_descriptor_set: &Option<ManagedAccelerationStructureDescriptorSet>,
    ) -> Result<Self> {
        let max_instances = resource_limits.max_draw_calls;

        let instances_geometry = instances_geometry(0);
        let mut sizes = AccelerationStructureBuildSizesInfoKHR::default();
        unsafe {
            context.device.get_acceleration_structure_build_sizes(
                AccelerationStructureBuildTypeKHR::DEVICE,
                &tlas_build_geometry_info(slice::from_ref(&instances_geometry)),
                &[max_instances],
                &mut sizes,
            );
        }

        let Some(factory) = &resource_factories.acceleration_structure_factory else {
            bail!("Acceleration structure factory is missing")
        };

        let acceleration_structure = factory.allocate(
            &resource_factories.buffer_factory,
            "tlas",
            sizes.acceleration_structure_size,
            AccelerationStructureTypeKHR::TOP_LEVEL,
        )?;

        let Some(descriptor_set) = acceleration_structures_descriptor_set else {
            bail!("Acceleration structure descriptor set is missing")
        };

        descriptor_set.write(frame_index, acceleration_structure.handle);

        let scratch = resource_factories.buffer_factory.create_managed_buffer(
            "tlas_scratch",
            sizes.build_scratch_size + context.properties.min_scratch_offset_alignment as DeviceSize,
            BufferUsageFlags::STORAGE_BUFFER,
            MemoryLocation::GpuOnly,
        )?;

        Ok(Self {
            acceleration_structure,
            scratch,

            alignment: context.properties.min_scratch_offset_alignment as DeviceSize,

            last_built_count: AtomicU32::new(u32::MAX),
            updates_since_rebuild: AtomicU32::new(0),
        })
    }

    pub fn scratch_address(&self) -> DeviceAddress {
        align_up(self.scratch.device_address, self.alignment)
    }

    pub fn next_build_mode(&self, instance_count: u32) -> BuildAccelerationStructureModeKHR {
        let rebuild = self.last_built_count.load(Ordering::Relaxed) != instance_count
            || self.updates_since_rebuild.load(Ordering::Relaxed) >= REBUILD_INTERVAL;

        if rebuild {
            self.last_built_count
                .store(instance_count, Ordering::Relaxed);
            self.updates_since_rebuild.store(0, Ordering::Relaxed);
            BuildAccelerationStructureModeKHR::BUILD
        } else {
            self.updates_since_rebuild.fetch_add(1, Ordering::Relaxed);
            BuildAccelerationStructureModeKHR::UPDATE
        }
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        let Some(factory) = &resource_factories.acceleration_structure_factory else {
            bail!("Acceleration structure factory is missing")
        };

        factory.destroy(&resource_factories.buffer_factory, self.acceleration_structure)?;

        resource_factories.buffer_factory.destroy_buffer(self.scratch)?;

        Ok(())
    }
}

pub fn instances_geometry(
    data_address: DeviceAddress,
) -> AccelerationStructureGeometryKHR<'static> {
    AccelerationStructureGeometryKHR::default()
        .geometry_type(GeometryTypeKHR::INSTANCES)
        .geometry(AccelerationStructureGeometryDataKHR {
            instances: AccelerationStructureGeometryInstancesDataKHR::default().data(
                DeviceOrHostAddressConstKHR {
                    device_address: data_address,
                },
            ),
        })
}

pub fn tlas_build_geometry_info<'a>(
    geometries: &'a [AccelerationStructureGeometryKHR<'a>],
) -> AccelerationStructureBuildGeometryInfoKHR<'a> {
    AccelerationStructureBuildGeometryInfoKHR::default()
        .ty(AccelerationStructureTypeKHR::TOP_LEVEL)
        .flags(
            BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE
                | BuildAccelerationStructureFlagsKHR::ALLOW_UPDATE,
        )
        .mode(BuildAccelerationStructureModeKHR::BUILD)
        .geometries(geometries)
}
