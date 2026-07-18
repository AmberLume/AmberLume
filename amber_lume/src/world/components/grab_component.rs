use shipyard::Component;
use physics::ObjectGrab;

#[derive(Component, Debug, Clone, Copy)]
pub struct GrabComponent {
    pub params: GrabParams,

    pub grab: Option<ObjectGrab>,
}

#[derive(Debug, Clone, Copy)]
pub struct GrabParams {
    pub distance: f32,

    pub linear_acceleration: f32,
    pub linear_stiffness: f32,
    pub linear_damping: f32,

    pub angular_acceleration: f32,
    pub angular_stiffness: f32,
    pub angular_damping: f32,
}
