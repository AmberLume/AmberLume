use crate::resources::dynamic::animation::animation_config::AnimationConfig;
use crate::resources::dynamic::res_ref::ResRef;
use shipyard::Component;
use std::sync::Arc;
use crate::resources::range_allocator::range_allocator::Allocation;

#[derive(Component)]
pub struct AnimationComponent {
    pub handle: Arc<ResRef>,

    pub bone_transform_allocation: Allocation,
}

#[derive(Component)]
pub struct AnimationBlueprintComponent {
    pub config: AnimationConfig,
}

impl AnimationBlueprintComponent {
    pub fn new(resource_key: &str) -> Self {
        Self {
            config: AnimationConfig::Alpaca {
                resource_key: resource_key.to_string(),
            },
        }
    }
}
