use std::marker::PhantomData;
use crate::render::vulkan::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::vulkan::factories::buffer::builder::into_buffer::IntoBuffer;
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;
use ash::vk::DeviceSize;
use crate::render::vulkan::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::vulkan::factories::buffer::chunk_buffer::chunk_buffer::ChunkBuffer;

pub struct ChunkBufferTag<B> {
    inner: B,

    chunk_size: DeviceSize,
}

impl<B, T> BufferBuilder<B, T> {
    pub fn chunked(self, count: u32) -> BufferBuilder<ChunkBufferTag<B>, T> {
        let size = self.size * count as DeviceSize;

        BufferBuilder {
            inner: ChunkBufferTag {
                inner: self.inner,
                
                chunk_size: self.size,
            },

            size,

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
        ChunkBuffer::handle(self.inner.into_buffer(handle), self.chunk_size)
    }
}
