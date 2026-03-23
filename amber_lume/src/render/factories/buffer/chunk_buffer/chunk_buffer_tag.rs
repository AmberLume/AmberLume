use std::marker::PhantomData;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::builder::into_buffer::IntoBuffer;
use crate::render::factories::buffer::managed_buffer::ManagedBuffer;
use ash::vk::DeviceSize;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::chunk_buffer::chunk_buffer::ChunkBuffer;

pub struct ChunkBufferTag<B> {
    inner: B,

    capacity: u32,
    chunk_size: DeviceSize,
}

impl<B, T> BufferBuilder<B, T> {
    pub fn chunked(self, capacity: u32) -> BufferBuilder<ChunkBufferTag<B>, T> {
        let total_size = self.total_size * capacity as DeviceSize;

        BufferBuilder {
            inner: ChunkBufferTag {
                inner: self.inner,
                
                capacity,
                chunk_size: self.total_size,
            },

            total_size,

            marker: PhantomData,
        }
    }
}

impl<T, B: IntoBuffer<T>> IntoBuffer<T> for ChunkBufferTag<B>
where
    B::Output: BufferInfo,
{
    type Output = ChunkBuffer<B::Output>;

    fn into_buffer(self, handle: ManagedBuffer) -> Self::Output {
        ChunkBuffer::handle(self.inner.into_buffer(handle), self.capacity, self.chunk_size)
    }
}
