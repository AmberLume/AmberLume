use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct PhysicsConfig {
    pub gravity: Vec3,
    pub fixed_delta_time: f32,

    pub paused: bool,
    pub interpolation: bool,
    pub render_colliders: bool,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            fixed_delta_time: 1.0 / 60.0,

            paused: false,
            interpolation: true,
            render_colliders: false,
        }
    }
}
