use math::{Quat, Vec3};

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
