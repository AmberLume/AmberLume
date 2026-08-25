use anyhow::bail;
use anyhow::Result;
use ash::vk::{AccessFlags, Buffer, BufferMemoryBarrier, DeviceAddress, DeviceSize};
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

    pub fn sub(&self, offset: DeviceSize, size: DeviceSize) -> Result<Self> {
        if offset + size > self.size {
            bail!(
                "Range '{}' sub {}..{} exceeds size {}",
                self.label,
                offset,
                offset + size,
                self.size,
            )
        }

        Ok(Self {
            label: self.label,

            buffer: self.buffer,
            offset: self.offset + offset,
            size,
            device_address: self.device_address + offset,

            mapped_ptr: if self.mapped_ptr.is_null() {
                self.mapped_ptr
            } else {
                unsafe { self.mapped_ptr.add(offset as usize) }
            },
        })
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

    pub fn barrier<'a>(&self, src_access_mask: AccessFlags, dst_access_mask: AccessFlags) -> BufferMemoryBarrier<'a> {
        BufferMemoryBarrier::default()
            .buffer(self.buffer)
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .offset(self.offset)
            .size(self.size)
    }
}
