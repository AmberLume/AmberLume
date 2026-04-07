use crate::resources::range_allocator::range_allocator_statistics::RangeAllocatorStatistics;

pub struct MeshBackendStatistics {
    pub index: RangeAllocatorStatistics,
    pub vertex: RangeAllocatorStatistics,
    pub submesh: RangeAllocatorStatistics,
}
