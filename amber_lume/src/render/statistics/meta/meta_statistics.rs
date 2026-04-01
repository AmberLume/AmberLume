use std::slice::from_raw_parts;
use ash::vk::{AccessFlags, BufferMemoryBarrier, BufferUsageFlags, DeviceSize};
use crate::render::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use bytemuck::Pod;
use gpu_allocator::MemoryLocation;
use crate::ids::{FrameIndex, SliceIndex};
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use anyhow::Result;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::view::buffer_view::BufferView;
use crate::render::pass::pass_context::PassContext;

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
                BufferUsageFlags::SHADER_DEVICE_ADDRESS
                    | BufferUsageFlags::STORAGE_BUFFER
                    | BufferUsageFlags::TRANSFER_DST,
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

    pub fn reset(&self, pass_context: &PassContext) -> BufferMemoryBarrier<'_> {
        let buffer_view = self.buffer
            .frame(pass_context.frame_index);

        let size = buffer_view.item_size() * self.capacity as DeviceSize;

        pass_context.clear_buffer(
            buffer_view,
            size,
            AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
        )
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        buffer_factory.destroy_buffer(self.buffer.into_managed_buffer())?;

        Ok(())
    }
}
