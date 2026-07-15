use shipyard::Component;
use crate::physics::ray_hit::RayHit;

#[derive(Component, Debug, Clone, Copy)]
pub struct FocusComponent {
    pub max_distance: f32,

    pub hit: Option<RayHit>,
}
