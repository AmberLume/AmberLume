use shipyard::Component;
use crate::world::physics::data::PhysicalBodyBlueprint;

#[derive(Component, Debug)]
pub struct PhysicalBodyBlueprintComponent {
    pub physical_body_blueprint: PhysicalBodyBlueprint,
}
