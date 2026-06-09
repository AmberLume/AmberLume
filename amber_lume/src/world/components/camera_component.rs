use glam::{Quat, Vec3};
use shipyard::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    Attached,
    Free,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct CameraComponent {
    pub mode: CameraMode,

    pub offset: Vec3,

    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,

    pub free_position: Vec3,
    pub fly_speed: f32,

    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl CameraComponent {
    pub fn local_rotation(&self) -> Quat {
        Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch)
    }
}
