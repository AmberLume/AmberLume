use ash::vk::{Buffer, DeviceSize};
use crate::render::vulkan::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::vulkan::factories::buffer::view::buffer_view::BufferView;

pub struct FrameBuffer<Inner: BufferInfo> {
    pub(in crate::render::vulkan::factories::buffer) inner: Inner,

    frame_size: DeviceSize,
}

impl<I: BufferInfo> FrameBuffer<I> {
    pub fn handle(inner: I, frame_size: DeviceSize) -> Self {
        Self {
            inner,

            frame_size,
        }
    }

    pub fn frame(&self, index: u32) -> BufferView<'_, I> {
        BufferView::create(
            &self.inner,
            self.frame_size * index as DeviceSize,
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
