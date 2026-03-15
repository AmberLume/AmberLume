use std::marker::PhantomData;
use ash::vk::{AccessFlags, Buffer, BufferMemoryBarrier, DeviceSize};
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;

pub struct TypedBuffer<T> {
    pub(in crate::render) handle: ManagedBuffer,

    pub(in crate::render) item_size: DeviceSize,

    pub(in crate::render) marker: PhantomData<T>,
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

impl<'a, T> BufferView<'a, TypedBuffer<T>> {
    pub fn item_size(&self) -> DeviceSize {
        self.inner.item_size
    }

    pub fn get(&self) -> BufferView<'a, ManagedBuffer> {
        BufferView {
            inner: &self.inner.handle,

            offset: self.offset,
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
