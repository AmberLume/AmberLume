use index_allocator::Allocation;
use index_allocator::ResourceId;
use resource_residency::ResRef;
use std::sync::Arc;

pub struct ManagedSkin {
    pub mesh_ids: Vec<ResourceId>,
    pub vertices_allocation: Allocation,
    pub submeshes_allocation: Allocation,
    pub bounds_allocation: Allocation,

    pub vertex_count: u32,
    pub source_vertex_offset: u32,
    pub source_skin_offset: u32,
    pub bone_count: u32,

    pub mesh: Arc<ResRef>,
    pub skeleton: Arc<ResRef>,
}
