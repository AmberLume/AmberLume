use ash::vk::{Buffer, DeviceSize};
use crate::render::vulkan::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::vulkan::factories::buffer::view::buffer_view::BufferView;

pub struct ChunkBuffer<Inner: BufferInfo> {
    inner: Inner,

    chunk_size: DeviceSize,
}

impl<I: BufferInfo> ChunkBuffer<I> {
    pub fn handle(inner: I, chunk_size: DeviceSize) -> Self {
        Self {
            inner,

            chunk_size,
        }
    }

    pub fn chunk(&self, index: u32) -> BufferView<'_, I> {
        BufferView::create(
            &self.inner,
            self.chunk_size * index as DeviceSize,
        )
    }
}

impl<Inner: BufferInfo> BufferInfo for ChunkBuffer<Inner> {
    fn handle(&self) -> Buffer { self.inner.handle() }

    fn entire_size(&self) -> DeviceSize { self.inner.entire_size() }
    
    fn into_managed_buffer(self) -> ManagedBuffer {
        self.inner.into_managed_buffer()
    }
}
