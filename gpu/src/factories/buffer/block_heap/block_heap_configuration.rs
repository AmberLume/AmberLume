use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu_allocator::MemoryLocation;

pub struct BlockHeapConfiguration {
    pub name: &'static str,
    pub block_size: DeviceSize,
    pub usage: BufferUsageFlags,
    pub location: MemoryLocation,
    pub frame_count: u32,
}
