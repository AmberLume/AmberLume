use glam::Quat;
use shipyard::Component;

#[derive(Component, Debug)]
pub struct RotationComponent {
    pub quaternion: Quat,
}
