use index_allocator::Allocation;
use resource_residency::ResRef;
use std::sync::Arc;

pub struct ManagedAnimation {
    pub name: String,
    pub skeleton: Arc<ResRef>,

    pub duration: f32,
    
    pub frames_allocation: Allocation,
}
