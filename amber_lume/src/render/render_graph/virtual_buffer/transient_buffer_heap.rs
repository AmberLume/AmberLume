use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;
use std::ptr::null_mut;
use gpu::FrameIndex;
use gpu::ManagedBuffer;
use gpu::ManagedBufferFactory;
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;

pub const TRANSIENT_BUFFER_ALIGNMENT: DeviceSize = 256;

pub fn align_up(value: DeviceSize, align: DeviceSize) -> DeviceSize {
    (value + align - 1) & !(align - 1)
}

pub struct TransientBufferHeap {
    buffer: ManagedBuffer,

    capacity_per_frame: DeviceSize,
    usage: BufferUsageFlags,
    frame_count: u32,

    region_start: DeviceSize,
}

impl TransientBufferHeap {
    pub fn create(
        buffer_factory: &ManagedBufferFactory,
        capacity_per_frame: DeviceSize,
        usage: BufferUsageFlags,
        frame_count: u32,
    ) -> Result<Self> {
        let buffer = buffer_factory.create_managed_buffer(
            "transient_gpu_heap",
            capacity_per_frame * frame_count as DeviceSize,
            usage,
            MemoryLocation::GpuOnly,
        )?;

        Ok(Self {
            buffer,

            capacity_per_frame,
            usage,
            frame_count,

            region_start: 0,
        })
    }

    pub fn matches(&self, capacity_per_frame: DeviceSize, usage: BufferUsageFlags, frame_count: u32) -> bool {
        self.capacity_per_frame == capacity_per_frame
            && self.usage == usage
            && self.frame_count == frame_count
    }

    pub fn begin_frame(&mut self, frame_index: FrameIndex) {
        assert!(
            frame_index.value < self.frame_count,
            "begin_frame index {} >= frame_count {}",
            frame_index.value, self.frame_count,
        );

        self.region_start = self.capacity_per_frame * frame_index.value as DeviceSize;
    }

    pub fn physical(&self, base_offset: DeviceSize, size: DeviceSize) -> PhysicalBuffer {
        let offset = self.region_start + base_offset;

        PhysicalBuffer::create(
            self.buffer.handle,
            offset,
            size,
            self.buffer.device_address + offset,
            null_mut(),
        )
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        buffer_factory.destroy_buffer(self.buffer)
    }
}
