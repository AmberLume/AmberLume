use shipyard::Component;

#[derive(Component)]
pub struct MeshBlueprintComponent {
    pub resource_key: String,
}

impl MeshBlueprintComponent {
    pub fn new(resource_key: String) -> Self {
        Self {
            resource_key,
        }
    }
}
