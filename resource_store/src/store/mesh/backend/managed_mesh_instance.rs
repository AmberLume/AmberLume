use crate::store::mesh::backend::geometry_range::GeometryRange;
use gpu_data::SubmeshGPU;
use index_allocator::Allocation;
use resource_residency::ResRef;
use std::sync::Arc;

pub struct ManagedMeshInstance {
    pub vertices_allocation: Allocation,
    pub submeshes_allocation: Allocation,
    pub bounds_allocation: Allocation,

    pub submeshes: Vec<SubmeshGPU>,
    pub geometry_ranges: Vec<GeometryRange>,
    pub vertex_slice_stride: u32,
    pub vertex_slice_count: u32,

    pub original: Arc<ResRef>,
}
