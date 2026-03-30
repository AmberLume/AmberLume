use std::sync::Arc;
use crate::resources::dynamic::mesh::mesh_config::MeshConfig;
use crate::resources::dynamic::res_ref::ResRef;
use shipyard::Component;

#[derive(Component)]
pub struct MeshComponent {
    pub config: MeshConfig,
    
    pub mesh_ref: Option<Arc<ResRef>>,
}

impl MeshComponent {
    pub fn new(resource_key: String) -> Self {
        let config = MeshConfig {
            resource_key,
        };

        Self { 
            config,

            mesh_ref: None,
        }
    }
}
