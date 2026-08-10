use resource_store::MeshConfig;
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
