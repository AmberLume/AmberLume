use shipyard::Component;
use physics::RayHit;

#[derive(Component, Debug, Clone, Copy)]
pub struct FocusComponent {
    pub max_distance: f32,

    pub hit: Option<RayHit>,
}
