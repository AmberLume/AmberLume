use crate::resources::common::resource_provider::ResourceProvider;
use crate::resources::model::model_backend::ModelBackend;
use crate::resources::resource_hub::ResourceHub;
use shipyard::Unique;
use std::sync::Arc;

#[derive(Unique)]
pub struct ResourceResolverUnique {
    pub model_provider: Arc<ResourceProvider<ModelBackend>>,
}

impl ResourceResolverUnique {
    pub fn new(resource_hub: Arc<ResourceHub>) -> Self {
        Self {
            model_provider: resource_hub.get_model_provider(),
        }
    }
}
