use ash::vk::{BuildAccelerationStructureFlagsKHR, BuildAccelerationStructureModeKHR, DeviceSize};
use gpu::ManagedAccelerationStructure;
use resource_store::GeometryRange;

pub struct BlasEntry {
    pub geometry_ranges: Vec<GeometryRange>,
    pub vertex_slice_stride: u32,

    pub flags: BuildAccelerationStructureFlagsKHR,
    pub acceleration_structure: ManagedAccelerationStructure,

    pub build_scratch_size: DeviceSize,
    pub update_scratch_size: DeviceSize,

    pub refits_until_build: u32,
}

impl BlasEntry {
    pub const REFITS_PER_BUILD: u32 = 60;

    pub fn refit_mode(&mut self) -> BuildAccelerationStructureModeKHR {
        if self.refits_until_build == 0 {
            self.refits_until_build = Self::REFITS_PER_BUILD;

            BuildAccelerationStructureModeKHR::BUILD
        } else {
            self.refits_until_build -= 1;

            BuildAccelerationStructureModeKHR::UPDATE
        }
    }
}
