use std::slice::from_raw_parts;
use std::sync::atomic::{AtomicU32, Ordering};
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceAddress, DeviceSize};
use bytemuck::Pod;
use gpu_allocator::MemoryLocation;
use index_allocator::{FrameIndex, SliceIndex};
use crate::factories::buffer::view::buffer_view::BufferView;
use crate::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::factories::buffer::builder::buffer_info::BufferInfo;
use crate::factories::buffer::managed_buffer::ManagedBuffer;
use crate::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;

pub struct Readback<T: Pod> {
    pub buffer: FrameBuffer<SliceBuffer<T>>,

    capacity: u32,
    frame_count: u32,
    frame: AtomicU32,
}

impl<T: Pod> Readback<T> {
    pub fn create(
        buffer_factory: &ManagedBufferFactory,
        label: &'static str,
        capacity: u32,
        frame_count: u32,
    ) -> Result<Self> {
        let buffer = BufferBuilder::slice::<T>(capacity.max(1))
            .per_frame(frame_count)
            .build(
                buffer_factory,
                label,
                BufferUsageFlags::STORAGE_BUFFER
                    | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                    | BufferUsageFlags::TRANSFER_DST,
                MemoryLocation::GpuToCpu,
            )?;

        Ok(Self {
            buffer,

            capacity,
            frame_count,
            frame: AtomicU32::new(0),
        })
    }

    pub fn begin_frame(&self, frame_index: FrameIndex) {
        self.frame.store(frame_index.value, Ordering::Relaxed);
    }

    pub fn frame_view(&self) -> BufferView<'_, SliceBuffer<T>> {
        self.buffer.frame(self.frame_index())
    }

    pub fn byte_size(&self) -> DeviceSize {
        self.frame_view().item_size() * self.capacity as DeviceSize
    }

    pub fn total_size(&self) -> DeviceSize {
        self.byte_size() * self.frame_count as DeviceSize
    }

    pub fn write_target(&self) -> DeviceAddress {
        self.slice().device_address()
    }

    pub fn values(&self) -> &[T] {
        let mapped_ptr = self.slice().mapped_ptr() as *const T;

        unsafe { from_raw_parts(mapped_ptr, self.capacity as usize) }
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        buffer_factory.destroy_buffer(self.buffer.into_managed_buffer())
    }

    fn frame_index(&self) -> FrameIndex {
        FrameIndex { value: self.frame.load(Ordering::Relaxed) }
    }

    fn slice(&self) -> BufferView<'_, ManagedBuffer> {
        self.frame_view().slice_at(SliceIndex::ZERO)
    }
}
