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
}
