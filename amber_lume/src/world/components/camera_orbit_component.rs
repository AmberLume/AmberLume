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
}
