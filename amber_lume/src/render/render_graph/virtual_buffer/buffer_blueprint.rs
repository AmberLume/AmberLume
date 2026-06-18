use ash::vk::{BufferUsageFlags, DeviceSize};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BufferBlueprint {
    pub size: DeviceSize,
    pub usage: BufferUsageFlags,
}

impl BufferBlueprint {
    pub fn new(size: DeviceSize, usage: BufferUsageFlags) -> Self {
        Self { size, usage }
    }

    pub fn storage(size: DeviceSize) -> Self {
        Self::new(size, BufferUsageFlags::STORAGE_BUFFER)
    }

    pub fn storage_dst(size: DeviceSize) -> Self {
        Self::new(size, BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST)
    }

    pub fn indirect(size: DeviceSize) -> Self {
        Self::new(size, BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::INDIRECT_BUFFER)
    }

    pub fn indirect_count(size: DeviceSize) -> Self {
        Self::new(
            size,
            BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::INDIRECT_BUFFER | BufferUsageFlags::TRANSFER_DST,
        )
    }
}
