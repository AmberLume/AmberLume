use ash::vk::{DeviceAddress, DeviceSize};
use ray_tracing::SkinnedBlasPlan;
use resource_store::GeometryRange;

pub struct SkinnedBLASBuild {
    pub geometry_ranges: Vec<GeometryRange>,
    pub vertex_address: DeviceAddress,
    pub plan: SkinnedBlasPlan,
    pub scratch_offset: DeviceSize,
}
