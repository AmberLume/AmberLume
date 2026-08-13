use ash::vk::{Buffer, DeviceSize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct BufferRegionKey {
    pub buffer: Buffer,
    pub offset: DeviceSize,
    pub size: DeviceSize,
}
