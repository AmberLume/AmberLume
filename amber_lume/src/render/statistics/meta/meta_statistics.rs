use std::any::Any;
use std::slice::from_raw_parts;
use std::sync::Arc;
use ash::vk::{AccessFlags, BufferMemoryBarrier, BufferUsageFlags, DeviceSize};
use gpu::{GpuMetaProvider, ResourceFactories};
use index_allocator::ArcUnwrapOrErr;
use gpu::FrameBuffer;
use gpu::SliceBuffer;
use bytemuck::Pod;
use gpu_allocator::MemoryLocation;
use index_allocator::FrameIndex;
use index_allocator::SliceIndex;
use gpu::BufferBuilder;
use gpu::ManagedBufferFactory;
use anyhow::Result;
use gpu::BufferInfo;
use gpu::BufferView;
use render_graph::FrameContext;

pub struct MetaStatistics<T: Pod> {
    buffer: FrameBuffer<SliceBuffer<T>>,

    capacity: u32,
}

impl<T: Pod> MetaStatistics<T> {
    pub fn new(
        label: &str,
        buffer_factory: &ManagedBufferFactory,
        capacity: u32,
        frame_count: u32,
    ) -> Result<Self> {
        let buffer = BufferBuilder::slice::<T>(capacity)
            .per_frame(frame_count)
            .build(
                &buffer_factory,
                label,
                BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST,
                MemoryLocation::GpuToCpu,
            )?;

        Ok(Self {
            buffer,

            capacity,
        })
    }

    pub fn buffer_view(&self, frame_index: FrameIndex) -> BufferView<'_, SliceBuffer<T>> {
        self.buffer.frame(frame_index)
    }

    pub fn collect(&self, frame_index: FrameIndex) -> &[T] {
        let buffer_view = self.buffer.frame(frame_index).slice_at(SliceIndex::ZERO);

        let mapped_ptr = buffer_view.mapped_ptr() as *const T;

        unsafe { from_raw_parts(mapped_ptr, self.capacity as usize) }
    }

    pub fn reset(&self, pass_context: &FrameContext) -> BufferMemoryBarrier<'_> {
        let buffer_view = self.buffer
            .frame(pass_context.frame_index);

        let size = buffer_view.item_size() * self.capacity as DeviceSize;

        pass_context.clear_buffer(
            buffer_view,
            size,
            AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
        )
    }

    pub fn host_read_barrier(&self, frame_index: FrameIndex) -> BufferMemoryBarrier<'_> {
        let buffer_view = self.buffer.frame(frame_index);

        buffer_view.barrier(
            AccessFlags::SHADER_WRITE,
            AccessFlags::HOST_READ,
        )
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        buffer_factory.destroy_buffer(self.buffer.into_managed_buffer())?;

        Ok(())
    }
}

impl<T: Pod + Send + Sync + 'static> GpuMetaProvider for MetaStatistics<T> {
    fn read(&self, frame_index: FrameIndex) -> Box<dyn Any + Send + Sync> {
        Box::new(self.collect(frame_index).to_vec())
    }

    fn destroy(self: Arc<Self>, resource_factories: &ResourceFactories) -> Result<()> {
        self.try_unwrap()?
            .destroy(&resource_factories.buffer_factory)
    }
}
