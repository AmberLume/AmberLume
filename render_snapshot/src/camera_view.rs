use gpu::{ProjectionMatrix, ViewMatrix};
use glam::{Quat, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct CameraView {
    pub position: Vec3,
    pub rotation: Quat,

    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl CameraView {
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

impl Default for CameraView {
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
