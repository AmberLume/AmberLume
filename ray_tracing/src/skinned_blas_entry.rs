use ash::vk::DeviceSize;
use gpu::ManagedAccelerationStructure;
use resource_store::SkinGeometry;

pub struct SkinnedBlasEntry {
    pub geometry: SkinGeometry,

    pub acceleration_structure: Option<ManagedAccelerationStructure>,

    pub build_scratch_size: DeviceSize,
    pub update_scratch_size: DeviceSize,

    pub updates_since_rebuild: u32,
}

impl SkinnedBlasEntry {
    pub const REBUILD_INTERVAL: u32 = 60;

    pub fn create(geometry: SkinGeometry) -> Self {
        Self {
            geometry,

            acceleration_structure: None,

            build_scratch_size: 0,
            update_scratch_size: 0,

            updates_since_rebuild: 0,
        }
    }
}
