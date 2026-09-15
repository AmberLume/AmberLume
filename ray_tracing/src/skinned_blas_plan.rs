use ash::vk::{AccelerationStructureKHR, BuildAccelerationStructureModeKHR, DeviceAddress, DeviceSize};

pub struct SkinnedBlasPlan {
    pub handle: AccelerationStructureKHR,
    pub device_address: DeviceAddress,
    pub mode: BuildAccelerationStructureModeKHR,
    pub scratch_size: DeviceSize,
}
