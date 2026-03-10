use std::marker::PhantomData;
use ash::vk::DeviceSize;
use crate::render::vulkan::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::vulkan::factories::buffer::builder::into_buffer::IntoBuffer;
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;
use crate::render::vulkan::factories::buffer::typed_buffer::typed_buffer::TypedBuffer;

pub struct TypedBufferTag<T> {
    marker: PhantomData<T>,

    pub(super) item_size: DeviceSize,
}

impl BufferBuilder<(), ()> {
    pub fn typed<T>() -> BufferBuilder<TypedBufferTag<T>, T> {
        let item_size = size_of::<T>() as DeviceSize;

        BufferBuilder {
            inner: TypedBufferTag {
                marker: PhantomData,
                
                item_size,
            },

            size: item_size,

            marker: PhantomData,
        }
    }
}

impl<T> IntoBuffer<T> for TypedBufferTag<T> {
    type Output = TypedBuffer<T>;

    fn into_buffer(self, handle: ManagedBuffer) -> TypedBuffer<T> {
        TypedBuffer {
            handle,

            item_size: self.item_size,

            marker: PhantomData,
        }
    }
}
