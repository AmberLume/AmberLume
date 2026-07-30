use ash::vk::{AccessFlags, Buffer, BufferMemoryBarrier, DeviceSize};
use crate::factories::buffer::builder::buffer_info::BufferInfo;
use crate::factories::buffer::managed_buffer::ManagedBuffer;
use crate::factories::buffer::view::buffer_view::BufferView;
use crate::ids::FrameIndex;

pub struct FrameBuffer<I: BufferInfo> {
    inner: I,

    capacity: u32,
    frame_size: DeviceSize,
}

impl<I: BufferInfo> FrameBuffer<I> {
    pub fn handle(inner: I, capacity: u32, frame_size: DeviceSize) -> Self {
        Self {
            inner,

            capacity,
            frame_size,
        }
    }

    pub fn frame(&self, index: FrameIndex) -> BufferView<'_, I> {
        assert!(
            index.value < self.capacity,
            "FrameBuffer::frame index {} out of bounds",
            index.value,
        );
        
        BufferView::create(
            &self.inner,
            self.frame_size * index.value as DeviceSize,
            self.frame_size,
        )
    }
}

impl<I: BufferInfo> BufferInfo for FrameBuffer<I> {
    fn handle(&self) -> Buffer { self.inner.handle() }

    fn entire_size(&self) -> DeviceSize { self.inner.entire_size() }

    fn into_managed_buffer(self) -> ManagedBuffer {
        self.inner.into_managed_buffer()
    }
}

impl<'a, I: BufferInfo> BufferView<'a, FrameBuffer<I>> {
    pub fn item_size(&self) -> DeviceSize {
        self.inner().frame_size 
    }

    pub fn frame(&self, index: FrameIndex) -> BufferView<'a, I> {
        assert!(
            index.value < self.inner().capacity,
            "FrameBuffer::frame index {} out of bounds",
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
            .size(self.item_size())
    }
}
