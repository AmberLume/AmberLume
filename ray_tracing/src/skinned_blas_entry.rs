use ash::vk::DeviceSize;
use gpu::ManagedAccelerationStructure;

pub struct SkinnedBlasEntry {
    pub acceleration_structure: ManagedAccelerationStructure,
    pub primitive_counts: Vec<u32>,

    pub build_scratch_size: DeviceSize,
    pub update_scratch_size: DeviceSize,

    pub updates_since_rebuild: u32,
}

impl SkinnedBlasEntry {
    pub const REBUILD_INTERVAL: u32 = 60;
}
