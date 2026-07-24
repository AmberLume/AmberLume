use glam::Vec3;
use shipyard::Component;

#[derive(Component, Debug, Clone, Copy)]
pub struct CameraOrbitComponent {
    pub pivot_offset: Vec3,

    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,

    pub collision_radius: f32,

    pub zoom_speed: f32,
    pub smoothing_speed: f32,

    pub current_distance: f32,
}

impl CameraOrbitComponent {
    pub fn create(
        pivot_offset: Vec3,
        distance: f32,
        min_distance: f32,
        max_distance: f32,
        collision_radius: f32,
        zoom_speed: f32,
        smoothing_speed: f32,
    ) -> Self {
        Self {
            pivot_offset,

            distance,
            min_distance,
            max_distance,

            collision_radius,

            zoom_speed,
            smoothing_speed,

            current_distance: distance,
        }
    }
}
