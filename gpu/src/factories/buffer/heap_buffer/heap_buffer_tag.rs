use std::marker::PhantomData;
use crate::factories::buffer::builder::into_buffer::IntoBuffer;
use crate::factories::buffer::managed_buffer::ManagedBuffer;
use ash::vk::DeviceSize;
use crate::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::factories::buffer::heap_buffer::heap_buffer::HeapBuffer;

pub struct HeapBufferTag {
    size: DeviceSize,
}

impl BufferBuilder<(), ()> {
    pub fn heap(size: DeviceSize) -> BufferBuilder<HeapBufferTag, ()> {
        BufferBuilder {
            inner: HeapBufferTag {
                size,
            },

            total_size: size,

            marker: PhantomData,
        }
    }
}

impl<T> IntoBuffer<T> for HeapBufferTag {
    type Output = HeapBuffer;

    fn into_buffer(self, handle: ManagedBuffer) -> Self::Output {
        HeapBuffer::handle(handle, self.size)
    }
}
