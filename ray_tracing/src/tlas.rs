use gpu::ResourceFactories;
use gpu::ManagedAccelerationStructure;
use gpu::ManagedAccelerationStructureDescriptorSet;
use gpu::RayTracingContext;
use anyhow::bail;
use anyhow::Result;
use ash::vk::{
    AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureBuildSizesInfoKHR,
    AccelerationStructureBuildTypeKHR, AccelerationStructureGeometryDataKHR,
    AccelerationStructureGeometryInstancesDataKHR, AccelerationStructureGeometryKHR,
    AccelerationStructureTypeKHR, BuildAccelerationStructureFlagsKHR,
    BuildAccelerationStructureModeKHR, DeviceAddress, DeviceOrHostAddressConstKHR,
    GeometryTypeKHR,
};
use index_allocator::ResourceLimits;
use std::slice;

pub struct TLAS {
    pub acceleration_structure: ManagedAccelerationStructure,
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

        Ok(Self {
            acceleration_structure,
        })
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        let Some(factory) = &resource_factories.acceleration_structure_factory else {
            bail!("Acceleration structure factory is missing")
        };

        factory.destroy(&resource_factories.buffer_factory, self.acceleration_structure)?;

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
        .flags(BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
        .mode(BuildAccelerationStructureModeKHR::BUILD)
        .geometries(geometries)
}
