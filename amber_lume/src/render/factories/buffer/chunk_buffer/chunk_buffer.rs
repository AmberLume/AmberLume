use ash::vk::{AccessFlags, Buffer, BufferMemoryBarrier, DeviceSize};
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;
use crate::ids::ChunkIndex;

pub struct ChunkBuffer<I: BufferInfo> {
    inner: I,

    capacity: u32,
    chunk_size: DeviceSize,
}

impl<I: BufferInfo> ChunkBuffer<I> {
    pub fn handle(inner: I, capacity: u32, chunk_size: DeviceSize) -> Self {
        Self {
            inner,

            capacity,
            chunk_size,
        }
    }

    pub fn chunk(&self, index: ChunkIndex) -> BufferView<'_, I> {
        assert!(
            index.value < self.capacity,
            "ChunkBuffer::chunk index {} out of bounds",
            index.value,
        );

        BufferView::create(
            &self.inner,
            self.chunk_size * index.value as DeviceSize,
            self.chunk_size,
        )
    }

    pub fn as_view(&self) -> BufferView<'_, ChunkBuffer<I>> {
        BufferView::create(
            &self,
            0,
            self.chunk_size * self.capacity as DeviceSize,
        )
    }
}

impl<I: BufferInfo> BufferInfo for ChunkBuffer<I> {
    fn handle(&self) -> Buffer { self.inner.handle() }

    fn entire_size(&self) -> DeviceSize { self.inner.entire_size() }
    
    fn into_managed_buffer(self) -> ManagedBuffer {
        self.inner.into_managed_buffer()
    }
}

impl<'a, T: BufferInfo> BufferView<'a, ChunkBuffer<T>> {
    pub fn item_size(&self) -> DeviceSize {
        self.inner().chunk_size
    }

    pub fn chunk(&self, index: ChunkIndex) -> BufferView<'a, T> {
        assert!(
            index.value < self.inner().capacity,
            "ChunkBuffer::chunk index {} out of bounds",
            index.value,
        );

        BufferView::create(
            &self.inner().inner,
            self.offset() + self.item_size() * index.value as DeviceSize,
            self.item_size(),
        )
    }

    pub fn barrier(
        &self,
        src_access_mask: AccessFlags,
        dst_access_mask: AccessFlags,
    ) -> BufferMemoryBarrier<'a> {
        BufferMemoryBarrier::default()
            .buffer(self.inner().handle())
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .offset(self.offset())
            .size(self.size())
    }
}
