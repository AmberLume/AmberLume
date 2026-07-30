use std::marker::PhantomData;
use ash::vk::{AccessFlags, Buffer, BufferMemoryBarrier, DeviceSize};
use crate::factories::buffer::builder::buffer_info::BufferInfo;
use crate::factories::buffer::managed_buffer::ManagedBuffer;
use crate::factories::buffer::view::buffer_view::BufferView;
use crate::ids::SliceIndex;

pub struct SliceBuffer<T> {
    handle: ManagedBuffer,

    capacity: u32,
    item_size: DeviceSize,

    marker: PhantomData<T>,
}

impl<T> SliceBuffer<T> {
    pub fn handle(handle: ManagedBuffer, capacity: u32, item_size: DeviceSize) -> Self {
        Self {
            handle,

            capacity,
            item_size,

            marker: PhantomData,
        }
    }

    pub fn slice_at(&self, index: SliceIndex) -> BufferView<'_, ManagedBuffer> {
        assert!(
            index.value < self.capacity,
            "SliceBuffer::slice_at index {} out of bounds",
            index.value,
        );

        BufferView::create(
            &self.handle,
            self.item_size * index.value as DeviceSize,
            self.item_size,
        )
    }
    
    pub fn slice_range(&self, index: SliceIndex, count: u32) -> BufferView<'_, ManagedBuffer> {
        assert!(
            index.value + count <= self.capacity,
            "SliceBuffer::slice_range {}..{} out of bounds, capacity {}",
            index.value,
            index.value + count,
            self.capacity,
        );

        BufferView::create(
            &self.handle,
            self.item_size * index.value as DeviceSize,
            self.item_size * count as DeviceSize,
        )
    }

    pub fn as_view(&self) -> BufferView<'_, SliceBuffer<T>> {
        BufferView::create(
            &self,
            0,
            self.item_size * self.capacity as DeviceSize,
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

impl<'a, I> BufferView<'a, SliceBuffer<I>> {
    pub fn item_size(&self) -> DeviceSize {
        self.inner().item_size
    }

    pub fn slice_at(&self, index: SliceIndex) -> BufferView<'a, ManagedBuffer> {
        assert!(
            index.value < self.inner().capacity,
            "BufferView::slice_at index {} out of bounds",
            index.value,
        );

        BufferView::create(
            &self.inner().handle,
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
            .offset(self.slice_at(SliceIndex::ZERO).offset())
            .size(self.size())
    }
}
