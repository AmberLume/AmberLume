use std::mem::ManuallyDrop;
use std::sync::Arc;
use ash::Device;
use anyhow::{bail, Result};
use ash::vk::{Buffer, BufferCreateInfo, BufferDeviceAddressInfo, BufferUsageFlags, DeviceAddress, DeviceSize, SharingMode};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use parking_lot::Mutex;
use tracing::info;
use crate::utils::debug_utils::DebugUtils;
use crate::factories::buffer::managed_buffer::ManagedBuffer;

pub struct ManagedBufferFactory {
    device: Device,
    allocator: Arc<Mutex<ManuallyDrop<Allocator>>>,
    debug_utils: Arc<DebugUtils>,
}

impl ManagedBufferFactory {
    pub fn create(
        device: Device,
        allocator: Arc<Mutex<ManuallyDrop<Allocator>>>,
        debug_utils: Arc<DebugUtils>,
    ) -> Self {
        Self {
            device,
            allocator,
            debug_utils,
        }
    }

    pub fn create_managed_buffer(
        &self,
        name: &str,
        size: DeviceSize,
        usage: BufferUsageFlags,
        location: MemoryLocation,
    ) -> Result<ManagedBuffer> {
        let handle = self.create_buffer(size, usage)?;

        let allocation = if let Ok(allocation) = self.create_buffer_allocation(handle, name, location) {
            allocation
        } else {
            unsafe { self.device.destroy_buffer(handle, None) };

            bail!("Failed to allocate buffer memory")
        };

        let device_address = self.get_buffer_device_address(handle);

        self.debug_utils.label(handle, &format!("buffer_{}", name));

        Ok(ManagedBuffer::create(
            name,
            handle,
            allocation,

            size,

            device_address,
        ))
    }

    fn create_buffer(
        &self,
        size: DeviceSize,
        usage: BufferUsageFlags,
    ) -> Result<Buffer> {
        let buffer_info = BufferCreateInfo::default()
            .size(size)
            .usage(usage | BufferUsageFlags::SHADER_DEVICE_ADDRESS)
            .sharing_mode(SharingMode::EXCLUSIVE);

        Ok(unsafe { self.device.create_buffer(&buffer_info, None)? })
    }

    fn create_buffer_allocation(
        &self,
        buffer: Buffer,
        label: &str,
        location: MemoryLocation,
    ) -> Result<Allocation> {
        let requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };

        let allocation = self.allocator.lock().allocate(&AllocationCreateDesc {
            name: label,
            requirements,
            location,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;

        unsafe { self.device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())? };

        Ok(allocation)
    }

    fn get_buffer_device_address(&self, buffer: Buffer) -> DeviceAddress {
        let address_info = BufferDeviceAddressInfo::default()
            .buffer(buffer);

        unsafe { self.device.get_buffer_device_address(&address_info) }
    }

    pub fn destroy_buffer(&self, buffer: ManagedBuffer) -> Result<()> {
        unsafe { self.device.destroy_buffer(buffer.handle, None) };

        self.allocator.lock().free(buffer.allocation)?;

        info!("ManagedBuffer '{}' destroyed", buffer.name);

        Ok(())
    }
}