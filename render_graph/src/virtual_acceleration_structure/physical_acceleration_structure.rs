use ash::vk::AccelerationStructureKHR;

#[derive(Clone, Copy)]
pub struct PhysicalAccelerationStructure {
    pub handle: AccelerationStructureKHR,
    pub descriptor_id: u32,
}
