use crate::factories::buffer::buffer_range::buffer_range::BufferRange;
use crate::factories::buffer::managed_buffer::ManagedBuffer;
use ash::vk::DeviceSize;
use index_allocator::RangeAllocator;
use std::marker::PhantomData;

pub struct RangeAllocation<T> {
    pub allocation: ManagedBuffer,
    pub allocator: RangeAllocator,

    marker: PhantomData<T>,
}

impl<T> RangeAllocation<T> {
    pub fn create(allocation: ManagedBuffer, allocator: RangeAllocator) -> Self {
        Self {
            allocation,
            allocator,

            marker: PhantomData,
        }
    }

    pub fn slice(&self, offset: u32, count: u32) -> BufferRange {
        let item_size = size_of::<T>() as DeviceSize;

        self.allocation.range(offset as DeviceSize * item_size, count as DeviceSize * item_size)
    }
}
