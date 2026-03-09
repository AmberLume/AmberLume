use std::marker::PhantomData;
use ash::vk::{Buffer, DeviceSize};
use crate::render::vulkan::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::vulkan::factories::buffer::builder::into_buffer::IntoBuffer;
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::vulkan::factories::buffer::slice_buffer::slice_buffer_tag::SliceBufferTag;
use crate::render::vulkan::factories::buffer::view::buffer_view::BufferView;

pub struct SliceBuffer<T> {
    pub(in crate::render::vulkan::factories::buffer) handle: ManagedBuffer,

    pub(in crate::render::vulkan::factories::buffer) item_size: DeviceSize,

    marker: PhantomData<T>,
}

impl<T> SliceBuffer<T> {
    pub fn at(&self, index: u32) -> BufferView<'_, ManagedBuffer> {
        BufferView::create(
            &self.handle,
            self.item_size * index as DeviceSize,
        )
    }
}

impl<T> BufferInfo for SliceBuffer<T> {
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

impl<T> IntoBuffer<T> for SliceBufferTag<T> {
    type Output = SliceBuffer<T>;

    fn into_buffer(self, handle: ManagedBuffer) -> SliceBuffer<T> {
        SliceBuffer {
            handle,

            item_size: self.item_size,

            marker: PhantomData,
        }
    }
}
