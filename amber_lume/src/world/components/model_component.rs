use crate::resources::model::model_config::ModelConfig;
use crate::resources::res_ref::ResRef;
use shipyard::Component;

#[derive(Component, Debug)]
pub struct ModelComponent {
    pub model_ref: ResRef<ModelConfig>,
}

impl ModelComponent {
    pub fn new(model_key: String) -> Self {
        let model_ref = ResRef::from(ModelConfig { name: model_key });

        Self { 
            model_ref,
        }
    }
}
