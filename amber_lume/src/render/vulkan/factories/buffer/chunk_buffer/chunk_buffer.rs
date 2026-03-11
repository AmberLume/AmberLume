use ash::vk::{AccessFlags, Buffer, BufferMemoryBarrier, DeviceSize};
use crate::render::vulkan::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::vulkan::factories::buffer::view::buffer_view::BufferView;
use crate::ids::ChunkIndex;

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

    pub fn chunk(&self, index: ChunkIndex) -> BufferView<'_, I> {
        BufferView::create(
            &self.inner,
            self.chunk_size * index.value as DeviceSize,
        )
    }

    pub fn barrier_entire<'a>(
        &self,
        src_access_mask: AccessFlags,
        dst_access_mask: AccessFlags,
    ) -> BufferMemoryBarrier<'a> {
        BufferMemoryBarrier::default()
            .buffer(self.handle())
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .offset(0)
            .size(self.entire_size())
    }
}

impl<Inner: BufferInfo> BufferInfo for ChunkBuffer<Inner> {
    fn handle(&self) -> Buffer { self.inner.handle() }

    fn entire_size(&self) -> DeviceSize { self.inner.entire_size() }
    
    fn into_managed_buffer(self) -> ManagedBuffer {
        self.inner.into_managed_buffer()
    }
}

impl<'a, T: BufferInfo> BufferView<'a, ChunkBuffer<T>> {
    pub fn item_size(&self) -> DeviceSize {
        self.inner.chunk_size
    }

    pub fn chunk(&self, chunk_index: ChunkIndex) -> BufferView<'a, T> {
        BufferView {
            inner: &self.inner.inner,

            offset: self.offset + self.item_size() * chunk_index.value as DeviceSize,
        }
    }

    pub fn barrier(
        &self,
        src_access_mask: AccessFlags,
        dst_access_mask: AccessFlags,
    ) -> BufferMemoryBarrier<'a> {
        BufferMemoryBarrier::default()
            .buffer(self.inner.handle())
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .offset(self.offset)
            .size(self.item_size())
    }
}
