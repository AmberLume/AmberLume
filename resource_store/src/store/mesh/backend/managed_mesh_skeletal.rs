use index_allocator::Allocation;
use resource_residency::ResRef;
use std::sync::Arc;

pub struct ManagedMeshSkeletal {
    pub vertex_skins_allocation: Allocation,
    pub bindings_allocation: Allocation,

    pub skeleton: Arc<ResRef>,
}
