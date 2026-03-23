use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::managed_buffer::ManagedBuffer;
use ash::vk::{Buffer, DeviceSize};
use crate::render::factories::buffer::view::buffer_view::BufferView;

pub struct FlatBuffer {
    handle: ManagedBuffer,

    size: DeviceSize,
}

impl FlatBuffer {
    pub fn handle(handle: ManagedBuffer, size: DeviceSize) -> Self {
        Self {
            handle,

            size,
        }
    }

    pub fn offset(&self, offset: DeviceSize) -> BufferView<'_, ManagedBuffer> {
        assert!(
            offset < self.size,
            "FlatBuffer::offset offset {} more than or equal to size {}",
            offset, self.size
        );

        BufferView::create(
            &self.handle,
            offset,
        )
    }
}

impl BufferInfo for FlatBuffer {
    fn handle(&self) -> Buffer {
        self.handle.handle
    }

    fn entire_size(&self) -> DeviceSize {
        self.handle.size
    }

    fn into_managed_buffer(self) -> ManagedBuffer {
        self.handle
    }
}

impl<'a> BufferView<'a, FlatBuffer> {
    pub fn offset(&self, offset: DeviceSize) -> BufferView<'a, ManagedBuffer> {
        assert!(
            offset < self.inner.size,
            "FlatBuffer::offset offset {} more than or equal to size {}",
            offset, self.inner.size
        );

        BufferView {
            inner: &self.inner.handle,

            offset: self.offset + offset
        }
    }
}
