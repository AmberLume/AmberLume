use std::sync::Arc;
use anyhow::Result;
use ash::khr::acceleration_structure::Device as AccelerationStructureDevice;
use ash::vk::{AccelerationStructureCreateInfoKHR, AccelerationStructureDeviceAddressInfoKHR, AccelerationStructureTypeKHR, BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::ray_tracing::managed_acceleration_structure::ManagedAccelerationStructure;
use crate::render::utils::debug_utils::DebugUtils;

pub struct AccelerationStructureFactory {
    as_loader: AccelerationStructureDevice,
    debug_utils: Arc<DebugUtils>,
}

impl AccelerationStructureFactory {
    pub fn new(
        as_loader: AccelerationStructureDevice,
        debug_utils: Arc<DebugUtils>,
    ) -> Self {
        Self {
            as_loader,
            debug_utils,
        }
    }

    pub fn allocate(
        &self,
        buffer_factory: &ManagedBufferFactory,
        label: &str,
        size: DeviceSize,
        acceleration_structure_type: AccelerationStructureTypeKHR,
    ) -> Result<ManagedAccelerationStructure> {
        let buffer = buffer_factory.create_managed_buffer(
            &format!("as_{label}"),
            size,
            BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
            MemoryLocation::GpuOnly,
        )?;

        let create_info = AccelerationStructureCreateInfoKHR::default()
            .buffer(buffer.handle)
            .offset(0)
            .size(size)
            .ty(acceleration_structure_type);

        let handle = unsafe { self.as_loader.create_acceleration_structure(&create_info, None)? };

        let device_address = unsafe {
            self.as_loader.get_acceleration_structure_device_address(
                &AccelerationStructureDeviceAddressInfoKHR::default().acceleration_structure(handle),
            )
        };

        self.debug_utils.label(handle, &format!("acceleration_structure_{label}"));

        Ok(ManagedAccelerationStructure {
            handle,
            buffer,
            device_address,
        })
    }

    pub fn destroy(
        &self,
        buffer_factory: &ManagedBufferFactory,
        acceleration_structure: ManagedAccelerationStructure,
    ) -> Result<()> {
        unsafe { self.as_loader.destroy_acceleration_structure(acceleration_structure.handle, None) };

        buffer_factory.destroy_buffer(acceleration_structure.buffer)
    }
}
