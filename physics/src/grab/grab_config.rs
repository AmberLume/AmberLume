#[derive(Debug, Clone, Copy)]
pub struct GrabConfig {
    pub linear_acceleration: f32,
    pub linear_stiffness: f32,
    pub linear_damping: f32,

    pub angular_acceleration: f32,
    pub angular_stiffness: f32,
    pub angular_damping: f32,
}
