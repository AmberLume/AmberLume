use ash::vk::{AccessFlags, Buffer, BufferMemoryBarrier, DeviceSize};
use crate::render::vulkan::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::vulkan::factories::buffer::view::buffer_view::BufferView;
use crate::ids::FrameIndex;

pub struct FrameBuffer<Inner: BufferInfo> {
    inner: Inner,

    frame_size: DeviceSize,
}

impl<I: BufferInfo> FrameBuffer<I> {
    pub fn handle(inner: I, frame_size: DeviceSize) -> Self {
        Self {
            inner,

            frame_size,
        }
    }

    pub fn frame(&self, index: FrameIndex) -> BufferView<'_, I> {
        BufferView::create(
            &self.inner,
            self.frame_size * index.value as DeviceSize,
        )
    }
}

impl<Inner: BufferInfo> BufferInfo for FrameBuffer<Inner> {
    fn handle(&self) -> Buffer { self.inner.handle() }

    fn entire_size(&self) -> DeviceSize { self.inner.entire_size() }

    fn into_managed_buffer(self) -> ManagedBuffer {
        self.inner.into_managed_buffer()
    }
}

impl<'a, T: BufferInfo> BufferView<'a, FrameBuffer<T>> {
    pub fn item_size(&self) -> DeviceSize {
        self.inner.frame_size 
    }

    pub fn frame(&self, frame_index: FrameIndex) -> BufferView<'a, T> {
        BufferView {
            inner: &self.inner.inner,

            offset: self.offset + self.item_size() * frame_index.value as DeviceSize,
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
