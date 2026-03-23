use std::marker::PhantomData;
use crate::render::factories::buffer::builder::into_buffer::IntoBuffer;
use crate::render::factories::buffer::flat_buffer::flat_buffer::FlatBuffer;
use crate::render::factories::buffer::managed_buffer::ManagedBuffer;
use ash::vk::DeviceSize;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;

pub struct FlatBufferTag {
    size: DeviceSize,
}

impl BufferBuilder<(), ()> {
    pub fn flat(size: DeviceSize) -> BufferBuilder<FlatBufferTag, ()> {
        BufferBuilder {
            inner: FlatBufferTag {
                size,
            },

            total_size: size,

            marker: PhantomData,
        }
    }
}

impl<T> IntoBuffer<T> for FlatBufferTag {
    type Output = FlatBuffer;

    fn into_buffer(self, handle: ManagedBuffer) -> FlatBuffer {
        FlatBuffer::handle(handle, self.size)
    }
}
