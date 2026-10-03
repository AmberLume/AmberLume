use crate::store::mesh::backend::geometry_range::GeometryRange;
use crate::store::mesh::backend::managed_mesh_skeletal::ManagedMeshSkeletal;
use gpu_data::SubmeshGPU;
use index_allocator::Allocation;
use resource_residency::ResRef;
use std::sync::Arc;

pub struct ManagedMesh {
    pub indices_allocation: Allocation,
    pub vertices_allocation: Allocation,
    pub uvs_allocation: Allocation,
    pub submeshes_allocation: Allocation,
    pub bounds_allocation: Allocation,

    pub submeshes: Vec<SubmeshGPU>,
    pub geometry_ranges: Vec<GeometryRange>,
    pub vertex_slice_stride: u32,
    pub vertex_slice_count: u32,

    pub skeletal: Option<ManagedMeshSkeletal>,
    pub materials: Vec<Arc<ResRef>>,
}
