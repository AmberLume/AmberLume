use gpu::ManagedBuffer;
use anyhow::bail;
use anyhow::Result;
use ash::vk::DeviceSize;
use index_allocator::FrameIndex;
use gpu::ManagedBufferFactory;
use crate::render::render_graph::virtual_buffer::heap_allocator_statistics::HeapAllocatorStatistics;
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;

pub struct HeapAllocator {
    buffer: ManagedBuffer,

    capacity_per_frame: DeviceSize,
    frame_count: u32,

    region_start: DeviceSize,
    head: DeviceSize,
}

impl HeapAllocator {
    pub fn create(
        buffer: ManagedBuffer,
        capacity_per_frame: DeviceSize,
        frame_count: u32,
    ) -> Result<Self> {
        let required = capacity_per_frame * frame_count as DeviceSize;
        if buffer.size < required {
            bail!(
                "HeapAllocator buffer {} smaller than required {}",
                buffer.size,
                required
            );
        }

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

        Ok(PhysicalBuffer::create(
            self.buffer.handle,
            aligned,
            size,
            self.buffer.device_address + aligned,
            unsafe { self.buffer.mapped_ptr().add(aligned as usize) } ,
        ))
    }

    pub fn allocate_for<T>(&mut self, count: usize) -> Result<PhysicalBuffer> {
        let size = (size_of::<T>() * count) as DeviceSize;
        let align = align_of::<T>() as DeviceSize;

        self.allocate(size, align)
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
