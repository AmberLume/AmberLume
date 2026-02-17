use std::sync::Arc;
use crate::resources::dynamic::model::model_config::ModelConfig;
use crate::resources::dynamic::res_ref::ResRef;
use shipyard::Component;

#[derive(Component)]
pub struct ModelComponent {
    pub config: ModelConfig,
    
    pub model_ref: Option<Arc<ResRef>>,
}

impl ModelComponent {
    pub fn new(model_key: String) -> Self {
        let config = ModelConfig { 
            name: model_key,
        };

        Self { 
            config,

            model_ref: None,
        }
    }
}
