use std::sync::Arc;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::range_allocator::range_allocator::Allocation;
use shipyard::Component;

#[derive(Component)]
pub struct SkeletonComponent {
    pub handle: Arc<ResRef>,

    pub bone_transform_allocation: Allocation,
}
