use gpu::ManagedAccelerationStructure;
use resource_store::GeometryRange;

pub struct BlasEntry {
    pub geometry_ranges: Vec<GeometryRange>,
    pub acceleration_structure: Option<ManagedAccelerationStructure>,
}
