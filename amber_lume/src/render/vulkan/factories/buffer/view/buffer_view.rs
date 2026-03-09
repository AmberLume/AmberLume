use ash::vk::{Buffer, DeviceAddress, DeviceSize};
use anyhow::Result;
use crate::render::vulkan::factories::buffer::flat_buffer::flat_buffer::FlatBuffer;
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::vulkan::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;

pub struct BufferView<'a, T> {
    inner: &'a T,

    offset: DeviceSize,
}

impl<'a, T> BufferView<'a, T> {
    pub fn create(inner: &'a T, offset: DeviceSize) -> Self {
        Self {
            inner,
            
            offset,
        }
    }
}

impl<'a, T> BufferView<'a, SliceBuffer<T>> {
    pub fn item_size(&self) -> DeviceSize {
        self.inner.item_size
    }
    
    pub fn at(&self, index: u32) -> BufferView<'a, ManagedBuffer> {
        BufferView {
            inner: &self.inner.handle,

            offset: self.offset + self.item_size() * index as DeviceSize,
        }
    }
}

impl<'a> BufferView<'a, FlatBuffer> {
    pub fn offset(&self, offset: DeviceSize) -> BufferView<'a, ManagedBuffer> {
        BufferView {
            inner: &self.inner.handle,

            offset: self.offset + offset
        }
    }
}

impl<'a> BufferView<'a, ManagedBuffer> {
    pub fn handle(&self) -> Buffer {
        self.inner.handle
    }
    
    pub fn device_address(&self) -> DeviceAddress {
        self.inner.device_address.unwrap() + self.offset()
    }

    pub fn offset(&self) -> DeviceSize {
        self.offset
    }

    pub fn stage<T>(&self, data: &[T]) -> Result<()> {
        self.inner.stage(self.offset, data)
    }

    pub fn mapped_ptr(&self) -> *mut u8 { 
        unsafe { self.inner.mapped_ptr().add(self.offset as usize) }
    }
}
