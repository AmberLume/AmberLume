use std::sync::Arc;
use resource_residency::ResRef;
use index_allocator::Allocation;
use shipyard::Component;

#[derive(Component)]
pub struct SkeletonComponent {
    pub handle: Arc<ResRef>,

    pub bone_transform_allocation: Allocation,
}
