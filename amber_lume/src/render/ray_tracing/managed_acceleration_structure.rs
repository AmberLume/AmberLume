use ash::vk::{AccelerationStructureKHR, DeviceAddress};
use crate::render::factories::buffer::managed_buffer::ManagedBuffer;

pub struct ManagedAccelerationStructure {
    pub handle: AccelerationStructureKHR,
    pub buffer: ManagedBuffer,
    pub device_address: DeviceAddress,
}
