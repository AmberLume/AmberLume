use glam::Vec3;
use shipyard::Component;

#[derive(Component, Debug)]
pub struct PositionComponent {
    pub position: Vec3,
}
