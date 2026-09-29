use index_allocator::RangeAllocatorStatistics;

pub struct MeshBackendStatistics {
    pub index: RangeAllocatorStatistics,
    pub vertex: RangeAllocatorStatistics,
    pub vertex_uv: RangeAllocatorStatistics,
    pub vertex_skin: RangeAllocatorStatistics,
    pub submesh: RangeAllocatorStatistics,
    pub submesh_bounds: RangeAllocatorStatistics,
}
