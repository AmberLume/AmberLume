use std::marker::PhantomData;
use ash::vk::DeviceSize;
use crate::render::vulkan::factories::buffer::builder::buffer_builder::BufferBuilder;

pub struct SliceBufferTag<T> {
    marker: PhantomData<T>,

    pub(super) item_size: DeviceSize,
}

impl BufferBuilder<(), ()> {
    pub fn slice<T>(capacity: u32) -> BufferBuilder<SliceBufferTag<T>, T> {
        let item_size = size_of::<T>() as DeviceSize;

        BufferBuilder {
            inner: SliceBufferTag {
                marker: PhantomData,
                
                item_size,
            },

            size: item_size * capacity as DeviceSize,

            marker: PhantomData,
        }
    }
}
