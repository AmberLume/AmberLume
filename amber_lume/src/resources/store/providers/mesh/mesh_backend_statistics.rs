use gpu::RangeAllocatorStatistics;

pub struct MeshBackendStatistics {
    pub index: RangeAllocatorStatistics,
    pub vertex: RangeAllocatorStatistics,
    pub submesh: RangeAllocatorStatistics,
}
