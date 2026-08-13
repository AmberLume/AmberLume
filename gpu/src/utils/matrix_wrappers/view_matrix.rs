use glam::{Mat4, Vec3};

#[derive(Clone, Copy)]
pub struct ViewMatrix {
    pub value: Mat4,
}

impl ViewMatrix {
    pub fn new(position: Vec3, target: Vec3) -> Self {
        Self {
            value: Mat4::look_at_rh(position, target, Vec3::Y),
        }
    }
}
