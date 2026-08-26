use crate::factories::acceleration_structure::managed_acceleration_structure::ManagedAccelerationStructure;
use crate::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::utils::debug_utils::DebugUtils;
use anyhow::Result;
use ash::khr::acceleration_structure::Device as AccelerationStructureDevice;
use ash::vk::{
    AccelerationStructureCreateInfoKHR, AccelerationStructureDeviceAddressInfoKHR,
    AccelerationStructureTypeKHR, BufferUsageFlags, DeviceSize,
};
use gpu_allocator::MemoryLocation;
use std::sync::Arc;
use tracing::info;

pub struct AccelerationStructureFactory {
    acceleration_structure_loader: AccelerationStructureDevice,
    debug_utils: Arc<DebugUtils>,
}

impl AccelerationStructureFactory {
    pub fn new(
        acceleration_structure_loader: AccelerationStructureDevice,
        debug_utils: Arc<DebugUtils>,
    ) -> Self {
        Self {
            acceleration_structure_loader,
            debug_utils,
        }
    }

    pub fn allocate(
        &self,
        buffer_factory: &ManagedBufferFactory,
        name: &str,
        size: DeviceSize,
        acceleration_structure_type: AccelerationStructureTypeKHR,
    ) -> Result<ManagedAccelerationStructure> {
        let buffer = buffer_factory.create_managed_buffer(
            &format!("acceleration_structure_{name}"),
            size,
            BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
            MemoryLocation::GpuOnly,
        )?;

        let create_info = AccelerationStructureCreateInfoKHR::default()
            .buffer(buffer.handle)
            .offset(0)
            .size(size)
            .ty(acceleration_structure_type);

        let handle = unsafe {
            self.acceleration_structure_loader
                .create_acceleration_structure(&create_info, None)?
        };

        let device_address = unsafe {
            self.acceleration_structure_loader
                .get_acceleration_structure_device_address(
                    &AccelerationStructureDeviceAddressInfoKHR::default()
                        .acceleration_structure(handle),
                )
        };

        self.debug_utils.label(handle, &format!("acceleration_structure_{name}"));

        Ok(ManagedAccelerationStructure::new(
            name,
            handle,
            buffer,
            device_address,
        ))
    }

    pub fn destroy(
        &self,
        buffer_factory: &ManagedBufferFactory,
        acceleration_structure: ManagedAccelerationStructure,
    ) -> Result<()> {
        unsafe {
            self.acceleration_structure_loader
                .destroy_acceleration_structure(acceleration_structure.handle, None)
        };
        
        info!("AccelerationStructure '{}' destroyed", acceleration_structure.name);

        buffer_factory.destroy_buffer(acceleration_structure.buffer)
    }
}
