use ash::vk::{Buffer, DeviceAddress, DeviceSize};

#[derive(Clone, Copy)]
pub struct PhysicalReadback {
    pub buffer: Buffer,
    pub offset: DeviceSize,
    pub size: DeviceSize,
    pub device_address: DeviceAddress,
    pub mapped_ptr: *const u8,
}
