use std::ptr::copy_nonoverlapping;
use anyhow::{Result, bail};
use ash::vk::{AccessFlags, Buffer, BufferMemoryBarrier, DeviceAddress, DeviceSize};
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

    pub fn stage<'a, T>(&self, offset: DeviceSize, data: &[T], dst_access_mask: AccessFlags) -> Result<BufferMemoryBarrier<'a>> {
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

        Ok(BufferMemoryBarrier::default()
            .buffer(self.handle)
            .src_access_mask(AccessFlags::HOST_WRITE)
            .dst_access_mask(dst_access_mask)
            .offset(offset)
            .size(size_of_val(data) as DeviceSize))
    }

    pub fn mapped_ptr(&self) -> *mut u8 {
        self.allocation.mapped_ptr()
            .unwrap()
            .as_ptr() as *mut u8
    }
}
