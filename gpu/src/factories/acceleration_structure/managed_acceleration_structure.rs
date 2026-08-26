use crate::factories::buffer::managed_buffer::ManagedBuffer;
use ash::vk::{AccelerationStructureKHR, DeviceAddress};

pub struct ManagedAccelerationStructure {
    pub name: String,
    pub handle: AccelerationStructureKHR,

    pub buffer: ManagedBuffer,

    pub device_address: DeviceAddress,
}

impl ManagedAccelerationStructure {
    pub fn new(
        name: &str,
        handle: AccelerationStructureKHR,
        buffer: ManagedBuffer,
        device_address: DeviceAddress,
    ) -> Self {
        Self {
            name: name.to_string(),
            handle,

            buffer,

            device_address,
        }
    }
}
