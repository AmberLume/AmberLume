use anyhow::bail;
use anyhow::Result;
use ash::vk::{Buffer, DeviceAddress, DeviceSize};
use std::ptr::copy_nonoverlapping;

#[derive(Clone, Copy)]
pub struct BufferRange {
    pub label: &'static str,

    pub buffer: Buffer,
    pub offset: DeviceSize,
    pub size: DeviceSize,
    pub device_address: DeviceAddress,

    pub mapped_ptr: *mut u8,
}

unsafe impl Send for BufferRange {}
unsafe impl Sync for BufferRange {}

impl BufferRange {
    pub fn create(
        label: &'static str,
        buffer: Buffer,
        offset: DeviceSize,
        size: DeviceSize,
        device_address: DeviceAddress,
        mapped_ptr: *mut u8,
    ) -> Self {
        Self {
            label,

            buffer,
            offset,
            size,
            device_address,

            mapped_ptr,
        }
    }

    pub fn write<T>(&self, data: &[T]) -> Result<()> {
        if data.len() == 0 {
            return Ok(());
        }
        if self.mapped_ptr.is_null() {
            bail!("Range '{}' is not host visible", self.label)
        }

        let bytes = size_of_val(data) as DeviceSize;

        if bytes > self.size {
            bail!(
                "Write {} bytes into range '{}' exceeds size {}",
                bytes,
                self.label,
                self.size,
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
