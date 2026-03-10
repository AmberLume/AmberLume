use crate::render::vulkan::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;
use ash::vk::{Buffer, DeviceSize};
use crate::render::vulkan::factories::buffer::view::buffer_view::BufferView;

pub struct FlatBuffer {
    pub(in crate::render::vulkan::factories::buffer) handle: ManagedBuffer,
}

impl FlatBuffer {
    pub fn offset(&self, offset: DeviceSize) -> BufferView<'_, ManagedBuffer> {
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
        BufferView {
            inner: &self.inner.handle,

            offset: self.offset + offset
        }
    }
}
