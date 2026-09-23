use index_allocator::Allocation;
use resource_residency::ResRef;
use std::sync::Arc;

pub struct ManagedMesh {
    pub indices_allocation: Allocation,
    pub vertices_allocation: Allocation,
    pub vertex_attributes_allocation: Allocation,
    pub vertex_skins_allocation: Option<Allocation>,
    pub bones_allocation: Option<Allocation>,
    pub submeshes_allocation: Allocation,

    pub skeleton: Option<Arc<ResRef>>,

    pub materials: Vec<Arc<ResRef>>,
}
