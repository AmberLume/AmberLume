use gpu::ManagedBuffer;
use anyhow::bail;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use ash::vk::DeviceSize;
use gpu_allocator::MemoryLocation;
use index_allocator::FrameIndex;
use gpu::ManagedBufferFactory;
use crate::virtual_buffer::heap_allocator_statistics::HeapAllocatorStatistics;
use crate::virtual_buffer::physical_buffer::PhysicalBuffer;

pub struct HeapAllocator {
    buffer: ManagedBuffer,

    capacity_per_frame: DeviceSize,
    frame_count: u32,

    region_start: DeviceSize,
    head: DeviceSize,
}

impl HeapAllocator {
    pub fn create(
        buffer_factory: &ManagedBufferFactory,
        capacity_per_frame: DeviceSize,
        frame_count: u32,
    ) -> Result<Self> {
        let buffer = buffer_factory.create_managed_buffer(
            "frame_heap",
            capacity_per_frame * frame_count as DeviceSize,
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::TRANSFER_DST
                | BufferUsageFlags::TRANSFER_SRC
                | BufferUsageFlags::VERTEX_BUFFER
                | BufferUsageFlags::INDEX_BUFFER,
            MemoryLocation::CpuToGpu,
        )?;

        Ok(Self {
            buffer,

            capacity_per_frame,
            frame_count,

            region_start: 0,
            head: 0,
        })
    }

    pub fn begin_frame(&mut self, frame_index: FrameIndex) {
        assert!(
            frame_index.value < self.frame_count,
            "begin_frame index {} >= frame_count {}",
            frame_index.value, self.frame_count,
        );

        self.region_start = self.capacity_per_frame * frame_index.value as DeviceSize;
        self.head = self.region_start;
    }

    pub fn allocate(&mut self, size: DeviceSize, align: DeviceSize) -> Result<PhysicalBuffer> {
        debug_assert!(align.is_power_of_two(), "align {} is not a power of two", align);

        let aligned = (self.head + align - 1) & !(align - 1);
        let end = aligned + size;
        let region_end = self.region_start + self.capacity_per_frame;

        if end > region_end {
            bail!(
                "overflow: requested {} at {} (aligned {}), region ends at {}",
                size, self.head, aligned, region_end,
            )
        }

        self.head = end;

        let range = self.buffer.range("frame_heap", aligned, size)?;

        Ok(PhysicalBuffer::create(range))
    }

    pub fn allocate_for_slice<T>(&mut self, data: &[T]) -> Result<PhysicalBuffer> {
        let size = (size_of::<T>() * data.len()) as DeviceSize;
        let align = align_of::<T>() as DeviceSize;

        self.allocate(size, align)
    }
    
    pub fn statistics(&self) -> HeapAllocatorStatistics {
        HeapAllocatorStatistics {
            capacity: self.capacity_per_frame as u32,
            used: (self.head - self.region_start) as u32,
        }
    }
    
    pub fn destroy(
        self,
        buffer_factory: &ManagedBufferFactory,
    ) -> Result<()> { 
        buffer_factory.destroy_buffer(self.buffer)
    }
}
