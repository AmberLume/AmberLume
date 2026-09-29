use crate::store::blas_queue::geometry_range::GeometryRange;
use gpu_data::SubmeshGPU;
use index_allocator::Allocation;
use resource_residency::ResRef;
use std::sync::Arc;

pub struct ManagedMesh {
    pub indices_allocation: Allocation,
    pub vertices_allocation: Allocation,
    pub uvs_allocation: Allocation,
    pub vertex_skins_allocation: Option<Allocation>,
    pub bones_allocation: Option<Allocation>,
    pub submeshes_allocation: Allocation,
    pub bounds_allocation: Allocation,

    pub submeshes: Vec<SubmeshGPU>,
    pub geometry_ranges: Vec<GeometryRange>,

    pub skeleton: Option<Arc<ResRef>>,

    pub materials: Vec<Arc<ResRef>>,
}
