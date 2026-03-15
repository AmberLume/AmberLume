use std::marker::PhantomData;
use ash::vk::{AccessFlags, Buffer, BufferMemoryBarrier, DeviceSize};
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::factories::buffer::view::buffer_view::BufferView;
use crate::ids::SliceIndex;

pub struct SliceBuffer<T> {
    pub(in crate::render) handle: ManagedBuffer,

    pub(in crate::render) item_size: DeviceSize,

    pub(in crate::render) marker: PhantomData<T>,
}

impl<T> SliceBuffer<T> {
    pub fn at(&self, index: SliceIndex) -> BufferView<'_, ManagedBuffer> {
        BufferView::create(
            &self.handle,
            self.item_size * index.value as DeviceSize,
        )
    }

    pub fn all(&self) -> BufferView<'_, ManagedBuffer> {
        self.at(SliceIndex { value: 0 })
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

impl<'a, T> BufferView<'a, SliceBuffer<T>> {
    pub fn item_size(&self) -> DeviceSize {
        self.inner.item_size
    }

    pub fn all(&self) -> BufferView<'a, ManagedBuffer> {
        self.at(SliceIndex { value: 0 })
    }

    pub fn at(&self, index: SliceIndex) -> BufferView<'a, ManagedBuffer> {
        BufferView {
            inner: &self.inner.handle,

            offset: self.offset + self.item_size() * index.value as DeviceSize,
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
            .offset(self.all().offset())
            .size(self.item_size())
    }
}
