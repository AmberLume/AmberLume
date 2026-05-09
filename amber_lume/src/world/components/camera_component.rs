use glam::{Quat, Vec3};
use shipyard::Component;

#[derive(Component, Debug, Clone, Copy)]
pub struct CameraComponent {
    pub offset: Vec3,
    pub rotation: Quat,

    pub fov: f32,
    pub near: f32,
    pub far: f32,
}
