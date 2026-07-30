use ash::vk::{AccessFlags, Buffer, BufferMemoryBarrier, DeviceAddress, DeviceSize};
use anyhow::Result;
use crate::factories::buffer::managed_buffer::ManagedBuffer;

pub struct BufferView<'a, I> {
    inner: &'a I,

    offset: DeviceSize,
    size: DeviceSize,
}

impl<'a, I> BufferView<'a, I> {
    pub fn create(inner: &'a I, offset: DeviceSize, size: DeviceSize) -> Self {
        Self {
            inner,
            
            offset,
            size,
        }
    }

    pub fn inner(&self) -> &'a I {
        self.inner
    }

    pub fn offset(&self) -> DeviceSize {
        self.offset
    }

    pub fn size(&self) -> DeviceSize {
        self.size
    }
}

impl<'a> BufferView<'a, ManagedBuffer> {
    pub fn handle(&self) -> Buffer {
        self.inner.handle
    }
    
    pub fn device_address(&self) -> DeviceAddress {
        self.inner.device_address + self.offset()
    }

    pub fn stage<T>(&self, data: &[T], dst_access_mask: AccessFlags) -> Result<BufferMemoryBarrier<'a>> {
        self.inner.stage(self.offset, data, dst_access_mask)
    }

    pub fn mapped_ptr(&self) -> *mut u8 { 
        unsafe { self.inner.mapped_ptr().add(self.offset as usize) }
    }
}
