use crate::factories::buffer::buffer_range::buffer_range::BufferRange;
use ash::vk::DeviceSize;
use index_allocator::SliceIndex;
use std::marker::PhantomData;

pub struct BufferArray<T> {
    range: BufferRange,

    capacity: u32,
    item_size: DeviceSize,

    marker: PhantomData<T>,
}

impl<T> Clone for BufferArray<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for BufferArray<T> {}

impl<T> BufferArray<T> {
    pub fn create(range: BufferRange, capacity: u32) -> Self {
        let item_size = size_of::<T>() as DeviceSize;

        assert!(
            capacity as DeviceSize * item_size <= range.size,
            "BufferArray of {} items does not fit range '{}' of {} bytes",
            capacity,
            range.label,
            range.size,
        );

        Self {
            range,

            capacity,
            item_size,

            marker: PhantomData,
        }
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn whole(&self) -> BufferRange {
        self.range
    }

    pub fn at(&self, index: SliceIndex) -> BufferRange {
        self.slice(index, 1)
    }

    pub fn slice(&self, index: SliceIndex, count: u32) -> BufferRange {
        assert!(
            index.value + count <= self.capacity,
            "BufferArray '{}' slice {}..{} out of bounds, capacity {}",
            self.range.label,
            index.value,
            index.value + count,
            self.capacity,
        );

        self.range
            .sub(
                self.item_size * index.value as DeviceSize,
                self.item_size * count as DeviceSize,
            )
            .expect("BufferArray slice within capacity must fit the range")
    }
}
