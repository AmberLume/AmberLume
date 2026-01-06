use glam::{Mat4, Vec3};
use shipyard::Unique;

#[derive(Debug, Clone)]
pub struct CameraStamp {
    pub position: Vec3,
    pub target: Vec3,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl CameraStamp {
    pub fn new(distance: f32, angle_ratio: f32, target: Vec3, fov: f32, near: f32, far: f32) -> Self {
        Self {
            position: Vec3::new(target.x, target.y + distance * angle_ratio, target.z + distance),
            target,
            fov,
            near,
            far,
        }
    }

    pub fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 2.0, 5.0),
            target: Vec3::ZERO,
            fov: 80.0,
            near: 0.1,
            far: 1000.0,
        }
    }

    pub fn to_view_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        let view_matrix = Mat4::look_at_rh(self.position, self.target, Vec3::Y);
        let mut projection_matrix = Mat4::perspective_rh(self.fov.to_radians(), aspect_ratio, self.near, self.far);

        projection_matrix.y_axis.y *= -1.0;

        projection_matrix * view_matrix
    }
}

#[derive(Unique, Debug)]
pub struct WorldCameraUnique {
    pub stamp: CameraStamp,
}

impl WorldCameraUnique {
    pub fn new() -> Self {
        Self {
            stamp: CameraStamp::default(),
        }
    }
}
