use glam::Vec3;
use shipyard::Unique;
use crate::utils::matrix_wrappers::{ProjectionMatrix, ViewMatrix};

#[derive(Unique, Debug, Clone, Copy)]
pub struct RenderViewUnique {
    pub camera_view: Camera,
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub target: Vec3,

    pub distance: f32,

    pub yaw: f32,
    pub pitch: f32,

    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn position(&self) -> Vec3 {
        let direction = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        );

        self.target - direction * self.distance
    }

    pub fn view(&self) -> ViewMatrix {
        ViewMatrix::new(self.position(), self.target)
    }
    
    pub fn projection(&self, aspect_ratio: f32) -> ProjectionMatrix {
        ProjectionMatrix::new(self.near, self.far, self.fov, aspect_ratio)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,

            distance: 6.0,

            yaw: -90.0_f32.to_radians(),
            pitch: -35_f32.to_radians(),

            fov: 80.0,
            near: 0.1,
            far: 100.0,
        }
    }
}

impl RenderViewUnique {
    pub fn new() -> Self {
        let camera_view = Camera::default();

        Self {
            camera_view,
        }
    }
}
