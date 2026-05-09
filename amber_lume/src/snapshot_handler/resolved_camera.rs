use crate::utils::matrix_wrappers::{ProjectionMatrix, ViewMatrix};
use glam::{Quat, Vec3};
use crate::world::components::camera_component::CameraComponent;

#[derive(Debug, Clone, Copy)]
pub struct ResolvedCamera {
    pub position: Vec3,
    pub rotation: Quat,

    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl ResolvedCamera {
    pub fn resolve(camera_component: &CameraComponent, parent_position: Vec3, parent_rotation: Quat) -> ResolvedCamera {
        ResolvedCamera {
            position: parent_position + parent_rotation * camera_component.offset,
            rotation: parent_rotation * camera_component.rotation,

            fov: camera_component.fov,
            near: camera_component.near,
            far: camera_component.far,
        }
    }

    pub fn forward(&self) -> Vec3 {
        (self.rotation * Vec3::Z).normalize()
    }

    pub fn view(&self) -> ViewMatrix {
        ViewMatrix::new(self.position, self.position + self.forward())
    }

    pub fn projection(&self, aspect_ratio: f32) -> ProjectionMatrix {
        ProjectionMatrix::new(self.near, self.far, self.fov, aspect_ratio)
    }
}

impl Default for ResolvedCamera {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,

            fov: 80.0,
            near: 0.1,
            far: 100.0,
        }
    }
}
