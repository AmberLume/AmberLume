use crate::factories::buffer::builder::buffer_info::BufferInfo;
use crate::factories::buffer::managed_buffer::ManagedBuffer;
use ash::vk::{Buffer, DeviceSize};
use crate::factories::buffer::view::buffer_view::BufferView;

pub struct HeapBuffer {
    handle: ManagedBuffer,

    size: DeviceSize,
}

impl HeapBuffer {
    pub fn handle(handle: ManagedBuffer, size: DeviceSize) -> Self {
        Self {
            handle,

            size,
        }
    }

    pub fn offset(&self, offset: DeviceSize) -> BufferView<'_, ManagedBuffer> {
        assert!(
            offset < self.size,
            "HeapBuffer::offset offset {} more than or equal to size {}",
            offset, self.size
        );

        BufferView::create(
            &self.handle,
            offset,
            self.size - offset,
        )
    }
}

impl BufferInfo for HeapBuffer {
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

impl<'a> BufferView<'a, HeapBuffer> {
    pub fn with_offset(&self, offset: DeviceSize) -> BufferView<'a, ManagedBuffer> {
        assert!(
            offset < self.inner().size,
            "HeapBuffer::offset offset {} more than or equal to size {}",
            offset, self.inner().size
        );

        BufferView::create(
            &self.inner().handle,
            self.offset() + offset,
            self.inner().size,
        )
    }
}
