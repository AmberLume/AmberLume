use anyhow::{Result, bail};
use ash::vk::{
    BufferCopy, BufferCreateInfo, BufferDeviceAddressInfo, BufferUsageFlags, CommandBuffer,
    DeviceAddress, DeviceSize, SharingMode,
};
use ash::{Device, vk};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts_mut;

pub struct Buffer {
    device: Device,

    pub handle: vk::Buffer,
    pub allocation: Allocation,
    pub size: DeviceSize,
    pub offset: DeviceSize,
    pub device_address: Option<DeviceAddress>,
}

impl Buffer {
    pub fn create(
        device: Device,
        allocator: &mut Allocator,
        size: DeviceSize,
        usage: BufferUsageFlags,
        location: MemoryLocation,
        name: &str,
    ) -> Result<Buffer> {
        let buffer_info = BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(SharingMode::EXCLUSIVE);

        let handle = unsafe { device.create_buffer(&buffer_info, None)? };

        let requirements = unsafe { device.get_buffer_memory_requirements(handle) };

        let allocation = allocator.allocate(&AllocationCreateDesc {
            name,
            requirements,
            location,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;
        unsafe { device.bind_buffer_memory(handle, allocation.memory(), allocation.offset())? };

        let device_address = if usage.contains(BufferUsageFlags::SHADER_DEVICE_ADDRESS) {
            let address_info = BufferDeviceAddressInfo::default().buffer(handle);
            let device_address = unsafe { device.get_buffer_device_address(&address_info) };

            Some(device_address)
        } else {
            None
        };

        Ok(Buffer {
            device,

            handle,
            allocation,
            size,
            offset: 0,
            device_address,
        })
    }

    pub fn copy_from_slice<T: Copy>(&mut self, data: &[T]) -> Result<()> {
        let size_bytes = size_of_val(data);
        if size_bytes > self.size as usize {
            bail!("Data exceeds buffer size")
        }

        let Some(ptr) = self.allocation.mapped_ptr() else {
            bail!("Buffer not host visible")
        };

        unsafe {
            copy_nonoverlapping(
                data.as_ptr() as *const u8,
                ptr.as_ptr() as *mut u8,
                size_bytes,
            )
        }

        Ok(())
    }

    pub fn copy_from_slice_staged<T: Copy>(
        &self,
        staging: &mut Buffer,
        command_buffer: CommandBuffer,
        data: &[T],
    ) -> Result<()> {
        staging.copy_from_slice(data)?;

        let region = BufferCopy::default().size(size_of_val(data) as DeviceSize);

        unsafe {
            self.device
                .cmd_copy_buffer(command_buffer, staging.handle, self.handle, &[region]);
        }

        Ok(())
    }

    pub fn mapped_slice<T>(&mut self) -> Option<&mut [T]> {
        let ptr = self.allocation.mapped_ptr()?;
        let count = self.size as usize / size_of::<T>();

        let data = unsafe { from_raw_parts_mut(ptr.as_ptr() as *mut T, count) };

        Some(data)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.handle, None);
        }
    }
}
