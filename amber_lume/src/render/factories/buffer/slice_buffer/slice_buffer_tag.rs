use std::marker::PhantomData;
use ash::vk::DeviceSize;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::builder::into_buffer::IntoBuffer;
use crate::render::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;

pub struct SliceBufferTag<T> {
    marker: PhantomData<T>,

    pub(in crate::render) item_size: DeviceSize,
}

impl BufferBuilder<(), ()> {
    pub fn slice<T>(capacity: u32) -> BufferBuilder<SliceBufferTag<T>, T> {
        let item_size = size_of::<T>() as DeviceSize;
        let size = item_size * capacity as DeviceSize;

        BufferBuilder {
            inner: SliceBufferTag {
                marker: PhantomData,

                item_size,
            },

            size,

            marker: PhantomData,
        }
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
