use crate::factories::buffer::buffer_range::buffer_range::BufferRange;
use crate::factories::buffer::managed_buffer::ManagedBuffer;
use ash::vk::DeviceSize;
use index_allocator::IndexManager;
use std::marker::PhantomData;
use std::sync::Arc;

pub struct SingleAllocation<T> {
    pub allocation: ManagedBuffer,
    pub allocator: Arc<IndexManager>,

    marker: PhantomData<T>,
}

impl<T> SingleAllocation<T> {
    pub fn create(allocation: ManagedBuffer, allocator: Arc<IndexManager>) -> Self {
        Self {
            allocation,
            allocator,

            marker: PhantomData,
        }
    }

    pub fn at(&self, index: u32) -> BufferRange {
        let item_size = size_of::<T>() as DeviceSize;

        self.allocation.range(index as DeviceSize * item_size, item_size)
    }
}
