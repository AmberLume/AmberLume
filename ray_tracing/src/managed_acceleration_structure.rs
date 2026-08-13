use ash::vk::{AccelerationStructureKHR, DeviceAddress};
use gpu::ManagedBuffer;

pub struct ManagedAccelerationStructure {
    pub handle: AccelerationStructureKHR,
    pub buffer: ManagedBuffer,
    pub device_address: DeviceAddress,
}
