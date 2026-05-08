use std::marker::PhantomData;
use ash::vk::{Buffer, DeviceSize};
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;

pub struct TypedBuffer<T> {
    handle: ManagedBuffer,

    item_size: DeviceSize,

    marker: PhantomData<T>,
}

impl<T> TypedBuffer<T> {
    pub fn handle(handle: ManagedBuffer, item_size: DeviceSize) -> Self {
        Self {
            handle,

            item_size,

            marker: PhantomData,
        }
    }
}

impl<T> BufferInfo for TypedBuffer<T> {
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

impl<'a, I> BufferView<'a, TypedBuffer<I>> {
    pub fn item_size(&self) -> DeviceSize {
        self.inner().item_size
    }

    pub fn get(&self) -> BufferView<'a, ManagedBuffer> {
        BufferView::create(
            &self.inner().handle,
            self.offset(),
            self.item_size(),
        )
    }
}
