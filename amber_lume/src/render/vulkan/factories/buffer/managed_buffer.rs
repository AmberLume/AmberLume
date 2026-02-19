use std::ptr::copy_nonoverlapping;
use anyhow::{Result, bail};
use ash::vk::{Buffer, DeviceAddress, DeviceSize};
use gpu_allocator::vulkan::Allocation;

pub struct ManagedBuffer {
    pub name: String,
    pub handle: Buffer,

    pub allocation: Allocation,

    pub size: DeviceSize,
    
    pub device_address: Option<DeviceAddress>,
}

impl ManagedBuffer {
    pub fn create(
        name: &str,
        handle: Buffer,
        allocation: Allocation,
        size: DeviceSize,
        device_address: Option<DeviceAddress>,
    ) -> Self {
        Self {
            name: name.to_string(),
            handle,
            allocation,
            size,
            device_address,
        }
    }

    pub fn stage<T>(&self, offset: DeviceSize, data: &[T]) -> Result<()> {
        let data_size = size_of_val(data) as DeviceSize;

        if offset + data_size > self.size {
            bail!("Data exceeds buffer size")
        }

        let Some(ptr) = self.allocation.mapped_ptr() else {
            bail!("Buffer not host visible")
        };

        unsafe {
            copy_nonoverlapping(
                data.as_ptr() as *const u8,
                (ptr.as_ptr() as *mut u8).add(offset as usize),
                data_size as usize,
            )
        }

        Ok(())
    }
}
