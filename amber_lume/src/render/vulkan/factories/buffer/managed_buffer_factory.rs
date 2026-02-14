use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};
use ash::Device;
use anyhow::{bail, Result};
use ash::vk::{Buffer, BufferCreateInfo, BufferDeviceAddressInfo, BufferUsageFlags, DeviceAddress, DeviceSize, SharingMode};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use crate::render::vulkan::debug_utils::DebugUtils;
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;

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

        let allocation = self.create_buffer_allocation(handle, name, location)?;
        let device_address = self.get_buffer_device_address(handle, usage);

        self.debug_utils.label(handle, &format!("managed_buffer_{}", name));

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
            .usage(usage)
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

        let allocation = if let Ok(allocator) = &mut self.allocator.lock() {
            allocator.allocate(&AllocationCreateDesc {
                name: label,
                requirements,
                location,
                linear: true,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            })?
        } else {
            bail!("Failed to lock allocator")
        };

        unsafe { self.device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())? };

        Ok(allocation)
    }

    fn get_buffer_device_address(
        &self,
        buffer: Buffer,
        usage: BufferUsageFlags,
    ) -> Option<DeviceAddress> {
        if usage.contains(BufferUsageFlags::SHADER_DEVICE_ADDRESS) {
            let address_info = BufferDeviceAddressInfo::default()
                .buffer(buffer);

            Some(unsafe { self.device.get_buffer_device_address(&address_info) })
        } else {
            None
        }
    }

    pub fn destroy(&self, buffer: ManagedBuffer) {
        unsafe { self.device.destroy_buffer(buffer.handle, None) };
    }
}