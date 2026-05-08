use anyhow::bail;
use anyhow::Result;
use ash::vk::{Buffer, DeviceAddress, DeviceSize};
use std::ptr::copy_nonoverlapping;

pub struct PhysicalBuffer {
    pub buffer: Buffer,
    pub offset: DeviceSize,
    pub size: DeviceSize,
    pub device_address: DeviceAddress,

    pub mapped_ptr: *mut u8,
}

impl PhysicalBuffer {
    pub fn create(
        buffer: Buffer,
        offset: DeviceSize,
        size: DeviceSize,
        device_address: DeviceAddress,
        mapped_ptr: *mut u8,
    ) -> Self {
        Self {
            buffer,
            offset,
            size,
            device_address,

            mapped_ptr,
        }
    }

    pub fn write<T>(&self, data: &[T]) -> Result<()> {
        let bytes = size_of_val(data) as DeviceSize;
        if bytes > self.size {
            bail!(
                "Write {} bytes exceeds allocation size {}",
                bytes,
                self.size
            );
        }

        unsafe {
            copy_nonoverlapping(
                data.as_ptr() as *const u8,
                self.mapped_ptr,
                bytes as usize,
            )
        }

        Ok(())
    }
}
