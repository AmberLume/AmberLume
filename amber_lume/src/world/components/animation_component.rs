use crate::resources::dynamic::animation::animation_config::AnimationConfig;
use crate::resources::dynamic::res_ref::ResRef;
use shipyard::Component;
use std::sync::Arc;

#[derive(Component)]
pub struct AnimationComponent {
    pub handle: Arc<ResRef>,
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
