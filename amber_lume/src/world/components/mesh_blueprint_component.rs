use crate::resources::store::providers::mesh::mesh_config::MeshConfig;
use shipyard::Component;

#[derive(Component)]
pub struct MeshBlueprintComponent {
    pub config: MeshConfig,
}

impl MeshBlueprintComponent {
    pub fn new(resource_key: String) -> Self {
        Self {
            config: MeshConfig::Alpaca { resource_key },
        }
    }
}
