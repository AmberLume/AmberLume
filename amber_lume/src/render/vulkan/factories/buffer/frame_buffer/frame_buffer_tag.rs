use std::marker::PhantomData;
use crate::render::vulkan::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::vulkan::factories::buffer::builder::into_buffer::IntoBuffer;
use crate::render::vulkan::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;
use ash::vk::DeviceSize;
use crate::render::vulkan::factories::buffer::builder::buffer_builder::BufferBuilder;

pub struct FrameBufferTag<B> {
    inner: B,

    frame_size: DeviceSize,
}

impl<T, B: IntoBuffer<T>> IntoBuffer<T> for FrameBufferTag<B>
where
    B::Output: BufferInfo,
{
    type Output = FrameBuffer<B::Output>;

    fn into_buffer(self, handle: ManagedBuffer) -> Self::Output {
        FrameBuffer::handle(self.inner.into_buffer(handle), self.frame_size)
    }
}

impl<B, T> BufferBuilder<B, T> {
    pub fn per_frame(self, frames: u32) -> BufferBuilder<FrameBufferTag<B>, T> {
        let size = self.size * frames as DeviceSize;

        BufferBuilder {
            inner: FrameBufferTag {
                inner: self.inner,
                
                frame_size: self.size,
            },

            size,

            marker: PhantomData,
        }
    }
}
