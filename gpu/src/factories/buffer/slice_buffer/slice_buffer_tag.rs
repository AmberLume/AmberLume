use std::marker::PhantomData;
use ash::vk::DeviceSize;
use crate::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::factories::buffer::builder::into_buffer::IntoBuffer;
use crate::factories::buffer::managed_buffer::ManagedBuffer;
use crate::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;

pub struct SliceBufferTag<T> {
    marker: PhantomData<T>,

    capacity: u32,
    item_size: DeviceSize,
}

impl BufferBuilder<(), ()> {
    pub fn slice<T>(capacity: u32) -> BufferBuilder<SliceBufferTag<T>, T> {
        let item_size = size_of::<T>() as DeviceSize;
        let total_size = item_size * capacity as DeviceSize;

        BufferBuilder {
            inner: SliceBufferTag {
                marker: PhantomData,

                capacity,
                item_size,
            },

            total_size,

            marker: PhantomData,
        }
    }
}

impl<T> IntoBuffer<T> for SliceBufferTag<T> {
    type Output = SliceBuffer<T>;

    fn into_buffer(self, handle: ManagedBuffer) -> SliceBuffer<T> {
        SliceBuffer::handle(handle, self.capacity, self.item_size)
    }
}
