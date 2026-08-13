use std::marker::PhantomData;
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;
use crate::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use anyhow::Result;
use crate::factories::buffer::builder::into_buffer::IntoBuffer;

pub struct BufferBuilder<B, T> {
    pub inner: B,

    pub total_size: DeviceSize,

    pub marker: PhantomData<T>,
}

impl<B, T> BufferBuilder<B, T> {
    pub fn build(
        self,
        factory: &ManagedBufferFactory,
        name: &str,
        usage: BufferUsageFlags,
        location: MemoryLocation,
    ) -> Result<B::Output> where
        B: IntoBuffer<T>,
    {
        let handle = factory.create_managed_buffer(name, self.total_size, usage, location)?;

        Ok(self.inner.into_buffer(handle))
    }
}
