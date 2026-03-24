use std::marker::PhantomData;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::builder::into_buffer::IntoBuffer;
use crate::render::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::factories::buffer::managed_buffer::ManagedBuffer;
use ash::vk::DeviceSize;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;

pub struct FrameBufferTag<I> {
    inner: I,

    capacity: u32,
    frame_size: DeviceSize,
}

impl<I, T> BufferBuilder<I, T> {
    pub fn per_frame(self, capacity: u32) -> BufferBuilder<FrameBufferTag<I>, T> {
        let total_size = self.total_size * capacity as DeviceSize;

        BufferBuilder {
            inner: FrameBufferTag {
                inner: self.inner,
                
                capacity,
                frame_size: self.total_size,
            },

            total_size,

            marker: PhantomData,
        }
    }
}

impl<T, I: IntoBuffer<T>> IntoBuffer<T> for FrameBufferTag<I>
where
    I::Output: BufferInfo,
{
    type Output = FrameBuffer<I::Output>;

    fn into_buffer(self, handle: ManagedBuffer) -> Self::Output {
        FrameBuffer::handle(self.inner.into_buffer(handle), self.capacity, self.frame_size)
    }
}
